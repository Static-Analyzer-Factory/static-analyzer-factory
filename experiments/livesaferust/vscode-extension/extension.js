const vscode = require('vscode');
const crypto = require('node:crypto');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { execFile } = require('node:child_process');
const { analyzeRustText } = require('./livesaferust-analyzer');

let diagnosticCollection;
let outputChannel;
let statusBar;
const timers = new Map();
const repoRoot = path.resolve(__dirname, '..', '..', '..');
const livesaferustRoot = path.join(repoRoot, 'experiments', 'livesaferust');
const runSafScript = path.join(livesaferustRoot, 'saf_integration', 'run_saf.py');
const pocScript = path.join(livesaferustRoot, 'livesaferust_poc.mjs');
const macPath = [
  '/opt/homebrew/bin',
  '/opt/homebrew/sbin',
  '/usr/local/bin',
  '/usr/local/sbin',
  '/usr/bin',
  '/bin',
  '/usr/sbin',
  '/sbin',
  path.join(os.homedir(), '.cargo', 'bin'),
].join(path.delimiter);

function isRustDocument(document) {
  return document && document.languageId === 'rust' && document.uri.scheme === 'file';
}

function config() {
  return vscode.workspace.getConfiguration('livesaferust');
}

function backend() {
  return config().get('backend', 'saf-backed');
}

function appendLog(message) {
  if (outputChannel) {
    outputChannel.appendLine(message);
  }
}

function setStatus(text, tooltip) {
  if (!statusBar) {
    return;
  }
  statusBar.text = text;
  statusBar.tooltip = tooltip;
  statusBar.show();
}

function toVscodeDiagnostic(item) {
  const range = new vscode.Range(
    item.range.start.line,
    item.range.start.character,
    item.range.end.line,
    item.range.end.character,
  );
  const diagnostic = new vscode.Diagnostic(
    range,
    item.message,
    vscode.DiagnosticSeverity.Warning,
  );
  diagnostic.code = item.code;
  diagnostic.source = item.source;
  if (item.fileName && item.trace && item.trace.length > 0) {
    diagnostic.relatedInformation = item.trace.map((label) => (
      new vscode.DiagnosticRelatedInformation(
        new vscode.Location(vscode.Uri.file(item.fileName), range),
        label,
      )
    ));
  }
  return diagnostic;
}

function scanDocumentFast(document) {
  if (!diagnosticCollection || !isRustDocument(document)) {
    return;
  }
  if (!config().get('enabled', true)) {
    diagnosticCollection.delete(document.uri);
    return;
  }

  const results = analyzeRustText(document.getText(), {
    fileName: document.uri.fsPath,
    uri: document.uri.toString(),
  });
  diagnosticCollection.set(document.uri, results.diagnostics.map(toVscodeDiagnostic));
}

function commandExists(commandPath) {
  try {
    fs.accessSync(commandPath, fs.constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

function resolvePython() {
  const candidates = [
    '/opt/homebrew/bin/python3',
    '/usr/local/bin/python3',
    '/usr/bin/python3',
  ];
  return candidates.find(commandExists) || 'python3';
}

function childEnv() {
  return {
    ...process.env,
    PATH: `${macPath}${path.delimiter}${process.env.PATH || ''}`,
  };
}

function execFilePromise(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    appendLog(`[LiveSafeRust] exec: ${command} ${args.join(' ')}`);
    execFile(command, args, {
      timeout: options.timeout || 180000,
      cwd: options.cwd || repoRoot,
      env: options.env || childEnv(),
      maxBuffer: 1024 * 1024 * 32,
    }, (error, stdout, stderr) => {
      if (stdout) {
        appendLog(stdout.trimEnd());
      }
      if (stderr) {
        appendLog(stderr.trimEnd());
      }
      if (error) {
        error.stdout = stdout;
        error.stderr = stderr;
        reject(error);
        return;
      }
      resolve({ stdout, stderr });
    });
  });
}

function stableDocumentId(document) {
  return crypto.createHash('sha256')
    .update(document.uri.toString())
    .digest('hex')
    .slice(0, 16);
}

function lspSeverityToVscode(severity) {
  if (severity === 1) return vscode.DiagnosticSeverity.Error;
  if (severity === 3) return vscode.DiagnosticSeverity.Information;
  if (severity === 4) return vscode.DiagnosticSeverity.Hint;
  return vscode.DiagnosticSeverity.Warning;
}

function lspDiagnosticToVscode(item, document) {
  const range = new vscode.Range(
    item.range.start.line,
    item.range.start.character,
    item.range.end.line,
    item.range.end.character,
  );
  const diagnostic = new vscode.Diagnostic(
    range,
    item.message,
    lspSeverityToVscode(item.severity),
  );
  diagnostic.code = item.code;
  diagnostic.source = item.source || 'LiveSafeRust/SAF';
  if (item.data?.sourceTrace?.length > 0) {
    diagnostic.relatedInformation = item.data.sourceTrace.map((step) => {
      const stepLine = Math.max(0, (step.line || 1) - 1);
      const stepColumn = Math.max(0, (step.column || 1) - 1);
      const stepRange = new vscode.Range(stepLine, stepColumn, stepLine, stepColumn + 1);
      return new vscode.DiagnosticRelatedInformation(
        new vscode.Location(document.uri, stepRange),
        `${step.kind}: ${step.message}`,
      );
    });
  } else {
    diagnostic.relatedInformation = (item.data?.trace || []).map((label) => (
      new vscode.DiagnosticRelatedInformation(
        new vscode.Location(document.uri, range),
        String(label),
      )
    ));
  }
  return diagnostic;
}

function rustcCompileDiagnostics(payload, document) {
  const stderr = payload?.compile?.stderr || payload?.saf?.error || '';
  const messageLine = stderr.split(/\r?\n/).find((line) => line.startsWith('error[') || line.startsWith('error:'));
  const locationLine = stderr.split(/\r?\n/).find((line) => line.includes('-->'));
  let line = 0;
  let character = 0;
  const location = locationLine?.match(/-->\s+.*:(\d+):(\d+)/);
  if (location) {
    line = Math.max(0, Number.parseInt(location[1], 10) - 1);
    character = Math.max(0, Number.parseInt(location[2], 10) - 1);
  }
  const range = new vscode.Range(
    line,
    character,
    line,
    Math.max(character + 1, document.lineAt(Math.min(line, document.lineCount - 1)).text.length),
  );
  const diagnostic = new vscode.Diagnostic(
    range,
    `LiveSafeRust could not run SAF because rustc failed: ${messageLine || 'compile failed'}`,
    vscode.DiagnosticSeverity.Error,
  );
  diagnostic.source = 'LiveSafeRust/rustc';
  diagnostic.code = 'compile-failed';
  return [diagnostic];
}

async function scanDocumentSafBacked(document) {
  if (!diagnosticCollection || !isRustDocument(document)) {
    return;
  }
  if (!config().get('enabled', true)) {
    diagnosticCollection.delete(document.uri);
    return;
  }

  const runId = stableDocumentId(document);
  const runDir = path.join(livesaferustRoot, 'out', 'vscode', runId);
  const sourcePath = path.join(runDir, path.basename(document.uri.fsPath || 'input.rs'));
  const factsPath = path.join(runDir, 'saf-facts.json');
  const previousFacts = path.join(runDir, 'previous-saf-facts.json');
  const safName = `vscode-${runId}`;
  const diagnosticsPath = path.join(livesaferustRoot, 'out', 'saf-backed', safName, 'diagnostics.json');

  fs.mkdirSync(runDir, { recursive: true });
  if (fs.existsSync(factsPath)) {
    fs.copyFileSync(factsPath, previousFacts);
  }
  fs.writeFileSync(sourcePath, document.getText(), 'utf8');

  const safArgs = [
    runSafScript,
    sourcePath,
    '--out-dir',
    runDir,
    '--out',
    factsPath,
    '--saf-image',
    config().get('safImage', 'llvm22'),
    '--crate-name',
    'livesaferust_vscode',
  ];
  const rustToolchain = config().get('rustToolchain', '');
  if (rustToolchain) {
    safArgs.push('--rustc', rustToolchain);
  }
  if (config().get('ensureSafSdk', false)) {
    safArgs.push('--ensure-saf-sdk');
  }
  if (fs.existsSync(previousFacts)) {
    safArgs.push('--previous', previousFacts);
  }

  setStatus('$(sync~spin) LiveSafeRust SAF', 'Running SAF-backed LiveSafeRust analysis');
  appendLog(`[LiveSafeRust] SAF-backed scan for ${document.uri.fsPath}`);
  appendLog(`[LiveSafeRust] temp input: ${sourcePath}`);
  appendLog(`[LiveSafeRust] PATH: ${childEnv().PATH}`);

  try {
    await execFilePromise(resolvePython(), safArgs, { cwd: repoRoot, timeout: 240000 });
    const pocArgs = [
      pocScript,
      '--workload',
      'saf-backed',
      '--facts',
      fs.existsSync(previousFacts) ? previousFacts : factsPath,
      '--saf-name',
      safName,
    ];
    if (fs.existsSync(previousFacts)) {
      pocArgs.push('--facts-edited', factsPath);
    }
    await execFilePromise(process.execPath, pocArgs, { cwd: repoRoot, timeout: 60000 });

    const payload = JSON.parse(fs.readFileSync(diagnosticsPath, 'utf8'));
    diagnosticCollection.set(
      document.uri,
      (payload.diagnostics || []).map((item) => lspDiagnosticToVscode(item, document)),
    );
    setStatus('$(check) LiveSafeRust', `SAF-backed scan complete: ${(payload.diagnostics || []).length} diagnostic(s)`);
  } catch (error) {
    if (fs.existsSync(factsPath)) {
      try {
        const failedPayload = JSON.parse(fs.readFileSync(factsPath, 'utf8'));
        if (failedPayload?.saf?.status === 'compile-failed') {
          diagnosticCollection.set(document.uri, rustcCompileDiagnostics(failedPayload, document));
          const compileMessage = failedPayload.compile?.stderr || failedPayload.saf?.error || 'rustc compile failed';
          appendLog(`[LiveSafeRust] rustc compile failed:\n${compileMessage}`);
          setStatus('$(error) LiveSafeRust', 'rustc compile failed before SAF');
          return;
        }
      } catch (parseError) {
        appendLog(`[LiveSafeRust] could not parse failed facts payload: ${parseError.message}`);
      }
    }
    const message = error.stderr || error.stdout || error.message || String(error);
    appendLog(`[LiveSafeRust] SAF-backed scan failed: ${message}`);
    vscode.window.showWarningMessage(
      `LiveSafeRust SAF-backed scan failed: ${String(message).split('\n')[0]}`,
    );
    setStatus('$(warning) LiveSafeRust', 'SAF-backed scan failed');
  }
}

function scanDocument(document, reason = 'manual') {
  if (backend() === 'saf-backed') {
    if (reason === 'change') {
      return;
    }
    if (reason === 'open' && !config().get('runSafOnOpen', false)) {
      diagnosticCollection?.delete(document.uri);
      return;
    }
    void scanDocumentSafBacked(document);
    return;
  }
  scanDocumentFast(document);
}

function scheduleScan(document) {
  if (!isRustDocument(document)) {
    return;
  }
  if (backend() === 'saf-backed') {
    return;
  }
  const key = document.uri.toString();
  const existing = timers.get(key);
  if (existing) {
    clearTimeout(existing);
  }
  const delay = config().get('debounceMs', 300);
  timers.set(key, setTimeout(() => {
    timers.delete(key);
    scanDocument(document);
  }, delay));
}

function activate(context) {
  diagnosticCollection = vscode.languages.createDiagnosticCollection('LiveSafeRust');
  outputChannel = vscode.window.createOutputChannel('LiveSafeRust');
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
  statusBar.command = 'livesaferust.rescan';
  setStatus('$(shield) LiveSafeRust', 'LiveSafeRust prototype diagnostics');
  context.subscriptions.push(diagnosticCollection);
  context.subscriptions.push(outputChannel);
  context.subscriptions.push(statusBar);

  context.subscriptions.push(vscode.commands.registerCommand('livesaferust.rescan', () => {
    const editor = vscode.window.activeTextEditor;
    if (editor && isRustDocument(editor.document)) {
      scanDocument(editor.document, 'manual');
    }
  }));
  context.subscriptions.push(vscode.commands.registerCommand('livesaferust.showOutput', () => {
    outputChannel?.show(true);
  }));

  context.subscriptions.push(vscode.workspace.onDidOpenTextDocument((document) => scanDocument(document, 'open')));
  context.subscriptions.push(vscode.workspace.onDidSaveTextDocument((document) => scanDocument(document, 'save')));
  context.subscriptions.push(vscode.workspace.onDidChangeTextDocument((event) => {
    scheduleScan(event.document);
  }));
  context.subscriptions.push(vscode.window.onDidChangeActiveTextEditor((editor) => {
    if (editor) {
      scanDocument(editor.document, 'open');
    }
  }));
  context.subscriptions.push(vscode.workspace.onDidChangeConfiguration((event) => {
    if (event.affectsConfiguration('livesaferust')) {
      for (const document of vscode.workspace.textDocuments) {
        scanDocument(document, 'open');
      }
    }
  }));

  for (const document of vscode.workspace.textDocuments) {
    scanDocument(document, 'open');
  }
}

function deactivate() {
  for (const timer of timers.values()) {
    clearTimeout(timer);
  }
  timers.clear();
  if (diagnosticCollection) {
    diagnosticCollection.dispose();
  }
}

module.exports = {
  activate,
  deactivate,
};
