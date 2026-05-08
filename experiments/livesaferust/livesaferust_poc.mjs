#!/usr/bin/env node
import crypto from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { performance } from 'node:perf_hooks';
import { fileURLToPath, pathToFileURL } from 'node:url';

const args = new Map();
for (let i = 2; i < process.argv.length; i += 1) {
  const arg = process.argv[i];
  if (arg.startsWith('--')) {
    const next = process.argv[i + 1];
    args.set(arg.slice(2), next && !next.startsWith('--') ? next : 'true');
    if (next && !next.startsWith('--')) i += 1;
  }
}

const scale = Number.parseInt(args.get('scale') ?? '800', 10);
const workloadArg = args.get('workload') ?? 'layered';
const chainDepth = Number.parseInt(args.get('chain-depth') ?? '24', 10);
const fanout = Number.parseInt(args.get('fanout') ?? '5', 10);
const synFactsPath = args.get('syn-facts');
const synEditedFactsPath = args.get('syn-facts-edited');
const synName = args.get('syn-name') ?? 'syn-facts';
const safFactsPath = args.get('facts');
const safEditedFactsPath = args.get('facts-edited');
const safName = args.get('saf-name') ?? 'saf-backed';
const root = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(root, 'out');
fs.mkdirSync(outDir, { recursive: true });

function sha(text) {
  return crypto.createHash('sha256').update(text).digest('hex').slice(0, 16);
}

function generateProject({ vulnerable = false, unrelatedEdit = false, helperCount = 800 }) {
  const helpers = [];
  for (let i = 0; i < helperCount; i += 1) {
    const name = `helper_${String(i).padStart(4, '0')}`;
    const bump = unrelatedEdit && i === helperCount - 1 ? 1 : 0;
    helpers.push(`fn ${name}(x: i32) -> i32 {`);
    helpers.push(`    x + ${i + bump}`);
    helpers.push('}');
    helpers.push('');
  }

  const prepareBody = vulnerable
    ? ['    s']
    : ['    sanitize_cmd(s)'];

  return [
    'use std::env;',
    'use std::process::Command;',
    '',
    ...helpers,
    'fn read_user() -> String {',
    '    env::args().nth(1).unwrap_or_default()',
    '}',
    '',
    'fn sanitize_cmd(s: String) -> String {',
    '    s.replace(";", "")',
    '}',
    '',
    'fn prepare_command(s: String) -> String {',
    ...prepareBody,
    '}',
    '',
    'fn run_command(cmd: String) {',
    '    Command::new(cmd).status().unwrap();',
    '}',
    '',
    'fn main() {',
    '    let raw = read_user();',
    '    let cmd = prepare_command(raw);',
    '    run_command(cmd);',
    '}',
    '',
  ].join('\n');
}

function generateLayeredProject({
  vulnerable = false,
  unrelatedEdit = false,
  helperCount = 800,
  depth = 24,
  fanoutCount = 5,
}) {
  const helpers = [];
  for (let i = 0; i < helperCount; i += 1) {
    const name = `helper_${String(i).padStart(4, '0')}`;
    const bump = unrelatedEdit && i === helperCount - 1 ? 1 : 0;
    helpers.push(`fn ${name}(x: i32) -> i32 {`);
    helpers.push(`    x + ${i + bump}`);
    helpers.push('}');
    helpers.push('');
  }

  const branchEntrypoints = [];
  const branchFunctions = [];
  for (let branch = 0; branch < fanoutCount; branch += 1) {
    const branchName = `b${String(branch).padStart(2, '0')}`;
    for (let i = 0; i < depth; i += 1) {
      const name = `${branchName}_wrap_${String(i).padStart(2, '0')}`;
      const callee = i === 0
        ? 'normalize_command'
        : `${branchName}_wrap_${String(i - 1).padStart(2, '0')}`;
      branchFunctions.push(`fn ${name}(s: String) -> String {`);
      branchFunctions.push(`    ${callee}(s)`);
      branchFunctions.push('}');
      branchFunctions.push('');
    }

    const entry = depth > 0
      ? `${branchName}_wrap_${String(depth - 1).padStart(2, '0')}`
      : 'normalize_command';
    const dispatch = `${branchName}_dispatch`;
    branchEntrypoints.push({ entry, dispatch });

    branchFunctions.push(`fn ${dispatch}(cmd: String) {`);
    branchFunctions.push('    run_command(cmd);');
    branchFunctions.push('}');
    branchFunctions.push('');
  }
  const normalizeBody = vulnerable
    ? ['    s']
    : ['    sanitize_cmd(s)'];

  return [
    'use std::env;',
    'use std::process::Command;',
    '',
    'unsafe extern "C" {',
    '    fn system(cmd: *const u8) -> i32;',
    '}',
    '',
    ...helpers,
    'fn read_user() -> String {',
    '    env::var("SAF_INPUT").unwrap_or_default()',
    '}',
    '',
    'fn sanitize_cmd(s: String) -> String {',
    '    s.replace(";", "")',
    '}',
    '',
    'fn normalize_command(s: String) -> String {',
    ...normalizeBody,
    '}',
    '',
    ...branchFunctions,
    'fn run_command(cmd: String) {',
    '    Command::new(cmd).status().unwrap();',
    '    unsafe { system(cmd.as_ptr()); }',
    '}',
    '',
    'fn main() {',
    '    let raw = read_user();',
    ...branchEntrypoints.flatMap(({ entry, dispatch }, index) => [
      `    let cmd_${String(index).padStart(2, '0')} = ${entry}(raw.clone());`,
      `    ${dispatch}(cmd_${String(index).padStart(2, '0')});`,
    ]),
    '}',
    '',
  ].join('\n');
}

function generateSuite(workload) {
  if (workload === 'flat') {
    return {
      name: 'flat',
      description: 'Mostly isolated helper functions plus a short taint chain.',
      clean: generateProject({ helperCount: scale }),
      vulnerable: generateProject({ vulnerable: true, helperCount: scale }),
      unrelatedEdit: generateProject({ unrelatedEdit: true, helperCount: scale }),
    };
  }

  if (workload === 'layered') {
    return {
      name: 'layered',
      description: 'A fan-out wrapper graph that forces summary-change propagation through several independent callers.',
      clean: generateLayeredProject({ helperCount: scale, depth: chainDepth, fanoutCount: fanout }),
      vulnerable: generateLayeredProject({
        vulnerable: true,
        helperCount: scale,
        depth: chainDepth,
        fanoutCount: fanout,
      }),
      unrelatedEdit: generateLayeredProject({
        unrelatedEdit: true,
        helperCount: scale,
        depth: chainDepth,
        fanoutCount: fanout,
      }),
    };
  }

  throw new Error(`Unknown workload "${workload}". Use flat, layered, or all.`);
}

function parseFunctions(source) {
  const lines = source.split(/\r?\n/);
  const functions = new Map();

  for (let i = 0; i < lines.length; i += 1) {
    const header = lines[i].match(/^\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/);
    if (!header) continue;

    const name = header[1];
    const paramsText = header[2].trim();
    const params = paramsText
      ? paramsText.split(',').map((part) => part.trim().split(':')[0].trim()).filter(Boolean)
      : [];

    let depth = 0;
    const body = [];
    const start = i;
    for (; i < lines.length; i += 1) {
      const line = lines[i];
      depth += (line.match(/{/g) ?? []).length;
      depth -= (line.match(/}/g) ?? []).length;
      body.push({ text: line, line: i + 1 });
      if (depth === 0) break;
    }

    functions.set(name, {
      name,
      params,
      body,
      startLine: start + 1,
      endLine: i + 1,
      text: body.map((entry) => entry.text).join('\n'),
      hash: sha(body.map((entry) => entry.text).join('\n')),
      calls: new Set(),
    });
  }

  for (const fn of functions.values()) {
    for (const entry of fn.body) {
      for (const match of entry.text.matchAll(/\b([A-Za-z_][A-Za-z0-9_]*)\s*\(/g)) {
        const callee = match[1];
        if (callee !== fn.name && functions.has(callee)) {
          fn.calls.add(callee);
        }
      }
    }
  }

  return functions;
}

function emptySummary() {
  return {
    returnDeps: [],
    sinkParams: [],
  };
}

function normalizeDeps(deps) {
  return [...deps].sort();
}

function summaryEquals(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

function splitArgs(text) {
  const args = [];
  let depth = 0;
  let start = 0;
  for (let i = 0; i < text.length; i += 1) {
    const ch = text[i];
    if (ch === '(') depth += 1;
    if (ch === ')') depth -= 1;
    if (ch === ',' && depth === 0) {
      args.push(text.slice(start, i).trim());
      start = i + 1;
    }
  }
  const tail = text.slice(start).trim();
  if (tail) args.push(tail);
  return args;
}

function findCalls(line, knownFunctions) {
  const calls = [];
  for (const name of knownFunctions) {
    const idx = line.indexOf(`${name}(`);
    if (idx < 0) continue;
    let open = idx + name.length;
    let depth = 0;
    for (let i = open; i < line.length; i += 1) {
      if (line[i] === '(') depth += 1;
      if (line[i] === ')') {
        depth -= 1;
        if (depth === 0) {
          calls.push({
            name,
            start: idx,
            end: i + 1,
            args: splitArgs(line.slice(open + 1, i)),
          });
          break;
        }
      }
    }
  }
  return calls;
}

function analyzeFunction(fn, summaries, knownFunctions) {
  const env = new Map();
  const findings = [];
  const sinkParams = new Set();
  let returnDeps = new Set();

  fn.params.forEach((param, index) => {
    env.set(param, new Set([`PARAM:${index}`]));
  });

  const evalExpr = (expr) => {
    const deps = new Set();
    if (/\bsanitize_cmd\s*\(/.test(expr) || /\bshell_escape\s*\(/.test(expr) || /\.replace\s*\(/.test(expr)) {
      return deps;
    }
    if (/\benv::(args|var)\s*\(/.test(expr) || /\bgetenv\s*\(/.test(expr)) {
      deps.add('SOURCE');
    }

    const calls = findCalls(expr, knownFunctions);
    let exprWithoutCalls = expr;
    for (const call of [...calls].sort((a, b) => b.start - a.start)) {
      exprWithoutCalls =
        exprWithoutCalls.slice(0, call.start) +
        ' '.repeat(call.end - call.start) +
        exprWithoutCalls.slice(call.end);
    }

    for (const [name, valueDeps] of env.entries()) {
      const re = new RegExp(`\\b${name}\\b`);
      if (re.test(exprWithoutCalls)) {
        for (const dep of valueDeps) deps.add(dep);
      }
    }

    for (const call of calls) {
      const summary = summaries.get(call.name) ?? emptySummary();
      const argDeps = call.args.map((arg) => evalExpr(arg));
      for (const dep of summary.returnDeps) {
        if (dep === 'SOURCE') deps.add('SOURCE');
        const paramMatch = dep.match(/^PARAM:(\d+)$/);
        if (paramMatch) {
          const index = Number.parseInt(paramMatch[1], 10);
          for (const argDep of argDeps[index] ?? []) deps.add(argDep);
        }
      }
    }

    return deps;
  };

  const noteSink = (line, lineNumber, sinkName, deps) => {
    const paramDeps = [...deps].filter((dep) => dep.startsWith('PARAM:'));
    for (const dep of paramDeps) {
      sinkParams.add(Number.parseInt(dep.slice('PARAM:'.length), 10));
    }
    if (deps.has('SOURCE')) {
      findings.push({
        ruleId: 'tainted-command',
        function: fn.name,
        line: lineNumber,
        sink: sinkName,
        message: `Tainted Rust input reaches ${sinkName}`,
        trace: ['env::args/env::var', fn.name, sinkName],
        sourceLine: line.trim(),
      });
    }
  };

  for (const entry of fn.body.slice(1, -1)) {
    const line = entry.text.replace(/\/\/.*$/, '').trim();
    if (!line) continue;

    const commandMatch = line.match(/Command::new\s*\(([^)]*)\)/);
    if (commandMatch) {
      noteSink(line, entry.line, 'Command::new', evalExpr(commandMatch[1]));
    }

    const systemMatch = line.match(/\bsystem\s*\(([^)]*)\)/);
    if (systemMatch) {
      noteSink(line, entry.line, 'system', evalExpr(systemMatch[1]));
    }

    for (const call of findCalls(line, knownFunctions)) {
      const summary = summaries.get(call.name) ?? emptySummary();
      call.args.forEach((arg, index) => {
        if (!summary.sinkParams.includes(index)) return;
        const deps = evalExpr(arg);
        noteSink(line, entry.line, `${call.name} -> sink`, deps);
      });
    }

    const letMatch = line.match(/^let\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*);$/);
    if (letMatch) {
      env.set(letMatch[1], evalExpr(letMatch[2]));
      continue;
    }

    const assignMatch = line.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*);$/);
    if (assignMatch) {
      env.set(assignMatch[1], evalExpr(assignMatch[2]));
      continue;
    }

    const retMatch = line.match(/^return\s+(.*);$/);
    if (retMatch) {
      returnDeps = evalExpr(retMatch[1]);
      continue;
    }

    if (!line.endsWith(';') && !line.startsWith('if ') && !line.startsWith('while ')) {
      returnDeps = evalExpr(line);
    }
  }

  return {
    summary: {
      returnDeps: normalizeDeps(returnDeps),
      sinkParams: [...sinkParams].sort((a, b) => a - b),
    },
    findings,
  };
}

function reverseCallGraph(functions) {
  const reverse = new Map();
  for (const name of functions.keys()) reverse.set(name, new Set());
  for (const fn of functions.values()) {
    for (const callee of fn.calls) {
      reverse.get(callee)?.add(fn.name);
    }
  }
  return reverse;
}

function analyzeProject(source, previous = null) {
  const start = performance.now();
  const functions = parseFunctions(source);
  const knownFunctions = [...functions.keys()];
  const reverse = reverseCallGraph(functions);
  const summaries = new Map(previous?.summaries ?? []);
  const findingsByFunction = new Map(previous?.findingsByFunction ?? []);
  const previousHashes = previous?.hashes ?? new Map();
  const hashes = new Map([...functions].map(([name, fn]) => [name, fn.hash]));

  const changed = [];
  for (const [name, fn] of functions) {
    if (previousHashes.get(name) !== fn.hash) changed.push(name);
  }

  const deleted = [...previousHashes.keys()].filter((name) => !functions.has(name));
  for (const name of deleted) {
    summaries.delete(name);
    findingsByFunction.delete(name);
  }

  const worklist = previous ? [...changed, ...deleted.flatMap((name) => [...(reverse.get(name) ?? [])])] : [...knownFunctions];
  const queued = new Set(worklist);
  const recomputed = new Set();
  let recomputeSteps = 0;
  let summaryChangedFunctions = 0;
  let summaryStableFunctions = 0;

  while (worklist.length > 0) {
    const name = worklist.shift();
    queued.delete(name);
    const fn = functions.get(name);
    if (!fn) continue;

    recomputed.add(name);
    recomputeSteps += 1;
    const oldSummary = summaries.get(name) ?? emptySummary();
    const { summary, findings } = analyzeFunction(fn, summaries, knownFunctions);
    const summaryChanged = !summaryEquals(oldSummary, summary);
    if (summaryChanged) {
      summaryChangedFunctions += 1;
    } else {
      summaryStableFunctions += 1;
    }
    summaries.set(name, summary);
    findingsByFunction.set(name, findings);

    if (summaryChanged) {
      for (const caller of reverse.get(name) ?? []) {
        if (!queued.has(caller)) {
          queued.add(caller);
          worklist.push(caller);
        }
      }
    }
  }

  const allFindings = [...findingsByFunction.values()].flat();
  const elapsedMs = performance.now() - start;
  return {
    source,
    functions,
    hashes,
    summaries,
    findingsByFunction,
    findings: dedupeFindings(allFindings),
    metrics: {
      functions: functions.size,
      changedFunctions: previous ? changed.length : functions.size,
      recomputedFunctions: previous ? recomputed.size : functions.size,
      cacheHits: previous ? Math.max(0, functions.size - recomputed.size) : 0,
      elapsedMs,
      recomputeSteps,
      summaryChangedFunctions,
      summaryStableFunctions,
    },
  };
}

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function uniqueFactNames(facts) {
  const seen = new Map();
  return facts.map((fact) => {
    const count = seen.get(fact.name) ?? 0;
    seen.set(fact.name, count + 1);
    return {
      ...fact,
      originalName: fact.name,
      uniqueName: count === 0 ? fact.name : `${fact.name}#${count + 1}`,
    };
  });
}

function resolveFactCall(rawCall, uniqueFactsByOriginalName, knownFunctions) {
  const exactMatches = uniqueFactsByOriginalName.get(rawCall) ?? [];
  if (exactMatches.length === 1) return exactMatches[0];

  const suffixMatches = [];
  for (const name of knownFunctions) {
    if (name.endsWith(`::${rawCall}`)) suffixMatches.push(name);
  }
  return suffixMatches.length === 1 ? suffixMatches[0] : null;
}

function factFileToFunctions(factFile) {
  const uniqueFacts = uniqueFactNames(factFile.functions);
  const knownFunctions = new Set(uniqueFacts.map((fn) => fn.uniqueName));
  const uniqueFactsByOriginalName = new Map();
  for (const fact of uniqueFacts) {
    const matches = uniqueFactsByOriginalName.get(fact.originalName) ?? [];
    matches.push(fact.uniqueName);
    uniqueFactsByOriginalName.set(fact.originalName, matches);
  }
  const functions = new Map();
  let resolvedCalls = 0;
  let unresolvedCalls = 0;
  let sourceHints = 0;
  let sinkHints = 0;

  for (const fact of uniqueFacts) {
    const calls = new Set();
    const rawCalls = fact.calls ?? [];
    for (const rawCall of rawCalls) {
      const resolved = resolveFactCall(rawCall, uniqueFactsByOriginalName, knownFunctions);
      if (resolved) {
        calls.add(resolved);
        resolvedCalls += 1;
      } else {
        unresolvedCalls += 1;
      }
    }

    sourceHints += fact.sourceHints?.length ?? 0;
    sinkHints += fact.sinkHints?.length ?? 0;
    functions.set(fact.uniqueName, {
      name: fact.uniqueName,
      originalName: fact.originalName,
      params: fact.params ?? [],
      body: [],
      startLine: fact.range?.startLine ?? 1,
      endLine: fact.range?.endLine ?? fact.range?.startLine ?? 1,
      text: '',
      hash: fact.hash,
      calls,
      rawCalls,
      sourceHints: fact.sourceHints ?? [],
      sinkHints: fact.sinkHints ?? [],
    });
  }

  return {
    functions,
    metrics: {
      resolvedCalls,
      unresolvedCalls,
      sourceHints,
      sinkHints,
    },
  };
}

function safFactsToFunctions(factFile) {
  const uniqueFacts = uniqueFactNames(factFile.functions ?? []);
  const knownFunctions = new Set(uniqueFacts.map((fn) => fn.uniqueName));
  const uniqueFactsByOriginalName = new Map();
  for (const fact of uniqueFacts) {
    const matches = uniqueFactsByOriginalName.get(fact.originalName) ?? [];
    matches.push(fact.uniqueName);
    uniqueFactsByOriginalName.set(fact.originalName, matches);
  }

  const functions = new Map();
  let resolvedCalls = 0;
  let unresolvedCalls = 0;
  let safLocalFindings = 0;

  for (const fact of uniqueFacts) {
    const calls = new Set();
    const rawCalls = fact.calls ?? [];
    for (const rawCall of rawCalls) {
      const resolved = resolveFactCall(rawCall, uniqueFactsByOriginalName, knownFunctions);
      if (resolved) {
        calls.add(resolved);
        resolvedCalls += 1;
      } else {
        unresolvedCalls += 1;
      }
    }

    const findings = fact.findings ?? [];
    safLocalFindings += findings.length;
    functions.set(fact.uniqueName, {
      name: fact.uniqueName,
      originalName: fact.originalName,
      params: fact.params ?? [],
      body: [],
      startLine: fact.range?.startLine ?? 1,
      endLine: fact.range?.endLine ?? fact.range?.startLine ?? 1,
      text: '',
      hash: fact.hash ?? sha(JSON.stringify(fact)),
      calls,
      rawCalls,
      safSummary: fact.summary ?? emptySummary(),
      safFindings: findings,
      isDeclaration: Boolean(fact.isDeclaration),
    });
  }

  return {
    functions,
    metrics: {
      resolvedCalls,
      unresolvedCalls,
      safLocalFindings,
      safRawFindingCount: factFile.saf?.rawFindingCount ?? factFile.findings?.length ?? 0,
      safDirectFindingCount: factFile.saf?.directFindingCount ?? factFile.findings?.length ?? 0,
      safInvoked: factFile.saf?.invoked === true ? 1 : 0,
      safCacheHit: factFile.saf?.cacheHit === true ? 1 : 0,
      safWallMs: factFile.timings?.safWallMs ?? 0,
      safProjectOpenMs: factFile.timings?.safProjectOpenMs ?? 0,
      safQueryMs: factFile.timings?.safQueryMs ?? 0,
      safOrchestrationMs: factFile.timings?.orchestrationMs ?? 0,
    },
  };
}

function allParamIndexes(fn) {
  return fn.params.map((_, index) => index);
}

function analyzeFactFunction(fn, summaries) {
  const returnDeps = new Set();
  const sinkParams = new Set();
  const findings = [];

  const hasLocalSource = fn.sourceHints.length > 0;
  if (hasLocalSource) {
    returnDeps.add('SOURCE');
  }

  const noteFactFinding = (sinkName, reason) => {
    findings.push({
      ruleId: 'tainted-command',
      function: fn.name,
      line: fn.startLine,
      sink: sinkName,
      message: `Possible tainted Rust input reaches ${sinkName}`,
      trace: [reason, fn.name, sinkName],
      sourceLine: `${fn.name} fact summary`,
    });
  };

  for (const sink of fn.sinkHints) {
    for (const index of allParamIndexes(fn)) sinkParams.add(index);
    if (hasLocalSource) {
      noteFactFinding(sink, 'local source hint');
    }
  }

  for (const callee of fn.calls) {
    const summary = summaries.get(callee) ?? emptySummary();
    const calleeReturnsSource =
      summary.returnDeps.includes('SOURCE') || summary.returnDeps.includes('TOP');
    const calleeReturnsParam = summary.returnDeps.some((dep) => dep.startsWith('PARAM:'));
    const calleeMaySink = summary.sinkParams.length > 0;

    if (calleeReturnsSource) {
      returnDeps.add('SOURCE');
    }
    if (calleeReturnsParam) {
      for (const index of allParamIndexes(fn)) returnDeps.add(`PARAM:${index}`);
    }
    if (calleeMaySink) {
      for (const index of allParamIndexes(fn)) sinkParams.add(index);
      if (hasLocalSource) {
        noteFactFinding(`${callee} -> sink`, 'local source hint');
      }
    }
  }

  return {
    summary: {
      returnDeps: normalizeDeps(returnDeps),
      sinkParams: [...sinkParams].sort((a, b) => a - b),
    },
    findings,
  };
}

function analyzeFactProject(factFile, previous = null) {
  const start = performance.now();
  const { functions, metrics: factMetrics } = factFileToFunctions(factFile);
  const knownFunctions = [...functions.keys()];
  const reverse = reverseCallGraph(functions);
  const summaries = new Map(previous?.summaries ?? []);
  const findingsByFunction = new Map(previous?.findingsByFunction ?? []);
  const previousHashes = previous?.hashes ?? new Map();
  const hashes = new Map([...functions].map(([name, fn]) => [name, fn.hash]));

  const changed = [];
  for (const [name, fn] of functions) {
    if (previousHashes.get(name) !== fn.hash) changed.push(name);
  }

  const deleted = [...previousHashes.keys()].filter((name) => !functions.has(name));
  for (const name of deleted) {
    summaries.delete(name);
    findingsByFunction.delete(name);
  }

  const worklist = previous ? [...changed, ...deleted.flatMap((name) => [...(reverse.get(name) ?? [])])] : [...knownFunctions];
  const queued = new Set(worklist);
  const recomputed = new Set();
  let recomputeSteps = 0;
  let summaryChangedFunctions = 0;
  let summaryStableFunctions = 0;

  while (worklist.length > 0) {
    const name = worklist.shift();
    queued.delete(name);
    const fn = functions.get(name);
    if (!fn) continue;

    recomputed.add(name);
    recomputeSteps += 1;
    const oldSummary = summaries.get(name) ?? emptySummary();
    const { summary, findings } = analyzeFactFunction(fn, summaries);
    const summaryChanged = !summaryEquals(oldSummary, summary);
    if (summaryChanged) {
      summaryChangedFunctions += 1;
    } else {
      summaryStableFunctions += 1;
    }
    summaries.set(name, summary);
    findingsByFunction.set(name, findings);

    if (summaryChanged) {
      for (const caller of reverse.get(name) ?? []) {
        if (!queued.has(caller)) {
          queued.add(caller);
          worklist.push(caller);
        }
      }
    }
  }

  const allFindings = [...findingsByFunction.values()].flat();
  const elapsedMs = performance.now() - start;
  return {
    source: factFile.source ?? '',
    functions,
    hashes,
    summaries,
    findingsByFunction,
    findings: dedupeFindings(allFindings),
    metrics: {
      functions: functions.size,
      changedFunctions: previous ? changed.length : functions.size,
      recomputedFunctions: previous ? recomputed.size : functions.size,
      cacheHits: previous ? Math.max(0, functions.size - recomputed.size) : 0,
      elapsedMs,
      recomputeSteps,
      summaryChangedFunctions,
      summaryStableFunctions,
      ...factMetrics,
    },
  };
}

function analyzeSafFunction(fn) {
  return {
    summary: {
      returnDeps: normalizeDeps(new Set(fn.safSummary?.returnDeps ?? [])),
      sinkParams: [...new Set(fn.safSummary?.sinkParams ?? [])].sort((a, b) => a - b),
    },
    findings: fn.safFindings ?? [],
  };
}

function analyzeSafProject(factFile, previous = null) {
  const start = performance.now();
  const { functions, metrics: factMetrics } = safFactsToFunctions(factFile);
  const knownFunctions = [...functions.keys()];
  const reverse = reverseCallGraph(functions);
  const summaries = new Map(previous?.summaries ?? []);
  const findingsByFunction = new Map(previous?.findingsByFunction ?? []);
  const previousHashes = previous?.hashes ?? new Map();
  const hashes = new Map([...functions].map(([name, fn]) => [name, fn.hash]));

  const changed = [];
  for (const [name, fn] of functions) {
    if (previousHashes.get(name) !== fn.hash) changed.push(name);
  }

  const deleted = [...previousHashes.keys()].filter((name) => !functions.has(name));
  for (const name of deleted) {
    summaries.delete(name);
    findingsByFunction.delete(name);
  }

  const worklist = previous ? [...changed, ...deleted.flatMap((name) => [...(reverse.get(name) ?? [])])] : [...knownFunctions];
  const queued = new Set(worklist);
  const recomputed = new Set();
  let recomputeSteps = 0;
  let summaryChangedFunctions = 0;
  let summaryStableFunctions = 0;

  while (worklist.length > 0) {
    const name = worklist.shift();
    queued.delete(name);
    const fn = functions.get(name);
    if (!fn) continue;

    recomputed.add(name);
    recomputeSteps += 1;
    const oldSummary = summaries.get(name) ?? emptySummary();
    const { summary, findings } = analyzeSafFunction(fn, summaries);
    const summaryChanged = !summaryEquals(oldSummary, summary);
    if (summaryChanged) {
      summaryChangedFunctions += 1;
    } else {
      summaryStableFunctions += 1;
    }
    summaries.set(name, summary);
    findingsByFunction.set(name, findings);

    if (summaryChanged) {
      for (const caller of reverse.get(name) ?? []) {
        if (!queued.has(caller)) {
          queued.add(caller);
          worklist.push(caller);
        }
      }
    }
  }

  const topLevelFindings = factFile.findings ?? [];
  const allFindings = [...topLevelFindings, ...findingsByFunction.values()].flat();
  const elapsedMs = performance.now() - start;
  return {
    source: factFile.input?.source ?? factFile.input?.llvmIr ?? '',
    functions,
    hashes,
    summaries,
    findingsByFunction,
    findings: dedupeFindings(allFindings),
    metrics: {
      functions: functions.size,
      changedFunctions: previous ? changed.length : functions.size,
      recomputedFunctions: previous ? recomputed.size : functions.size,
      cacheHits: previous ? Math.max(0, functions.size - recomputed.size) : 0,
      elapsedMs,
      recomputeSteps,
      summaryChangedFunctions,
      summaryStableFunctions,
      ...factMetrics,
    },
  };
}

function dedupeFindings(findings) {
  const seen = new Set();
  const out = [];
  for (const finding of findings) {
    const rawKey = finding.safFinding?.finding_id
      ?? `${finding.safFinding?.source_id ?? ''}:${finding.safFinding?.sink_id ?? ''}`;
    const key = rawKey === ':' || rawKey === ''
      ? `${finding.ruleId}:${finding.function}:${finding.line}:${finding.sink}`
      : `saf:${rawKey}`;
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(finding);
  }
  return out;
}

function diagnosticsFor(uri, findings) {
  return {
    uri,
    diagnostics: findings.map((finding) => {
      const span = finding.sourceSpan;
      const startLine = (span?.startLine ?? finding.line) - 1;
      const startColumn = Math.max(0, (span?.startColumn ?? 1) - 1);
      const endLine = (span?.endLine ?? span?.startLine ?? finding.line) - 1;
      const endColumn = Math.max(startColumn + 1, (span?.endColumn ?? 121) - 1);
      return {
        range: {
          start: { line: startLine, character: startColumn },
          end: { line: endLine, character: endColumn },
        },
        severity: 2,
        code: finding.ruleId,
        source: 'LiveSafeRust/SAF-POC',
        message: finding.message,
        data: {
          function: finding.function,
          sink: finding.sink,
          trace: finding.trace,
          sourceTrace: finding.sourceTrace,
          sourceSpan: span,
          rawSink: finding.safFinding?.sink_name ?? finding.safFinding?.sink_id,
        },
      };
    }),
  };
}

function semanticFacts(result, frontend = {}) {
  return {
    schema: 'livesaferust.semantic-facts/0.1',
    frontend: {
      kind: 'source-sidecar-poc',
      parser: 'lightweight-js',
      replaceableBy: ['syn-sidecar', 'rust-analyzer-hir'],
      ...frontend,
    },
    functions: [...result.functions.values()].map((fn) => ({
      name: fn.name,
      hash: fn.hash,
      params: fn.params,
      range: { startLine: fn.startLine, endLine: fn.endLine },
      calls: [...fn.calls].sort(),
      summary: result.summaries.get(fn.name) ?? emptySummary(),
    })),
  };
}

function writeJson(name, value) {
  fs.writeFileSync(path.join(outDir, name), `${JSON.stringify(value, null, 2)}\n`);
}

function writeJsonAt(dir, name, value) {
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, name), `${JSON.stringify(value, null, 2)}\n`);
}

function printSummary(workload, results, assertions) {
  const rows = [
    ['scenario', 'functions', 'changed', 'recomputed', 'sum Δ', 'sum =', 'cache hits', 'findings', 'ms'],
    ...results.map((r) => [
      r.name,
      r.result.metrics.functions,
      r.result.metrics.changedFunctions,
      r.result.metrics.recomputedFunctions,
      r.result.metrics.summaryChangedFunctions,
      r.result.metrics.summaryStableFunctions,
      r.result.metrics.cacheHits,
      r.result.findings.length,
      r.result.metrics.elapsedMs.toFixed(3),
    ]),
  ];
  console.log(`\nworkload: ${workload}`);
  const widths = rows[0].map((_, col) => Math.max(...rows.map((row) => String(row[col]).length)));
  for (const row of rows) {
    console.log(row.map((cell, col) => String(cell).padEnd(widths[col])).join('  '));
  }
  const status = assertions.every((assertion) => assertion.pass) ? 'PASS' : 'FAIL';
  console.log(`assertions: ${status}`);
  for (const assertion of assertions) {
    console.log(`  ${assertion.pass ? 'ok' : 'not ok'} - ${assertion.name}`);
  }
}

function buildAssertions(resultsByName, workload) {
  const cleanCold = resultsByName.get('clean cold');
  const unrelatedIncremental = resultsByName.get('unrelated incremental');
  const vulnerableIncremental = resultsByName.get('vulnerable incremental');
  const vulnerableFull = resultsByName.get('vulnerable full');
  const sameFindings =
    JSON.stringify(vulnerableIncremental.findings.map((f) => [f.ruleId, f.function, f.sink])) ===
    JSON.stringify(vulnerableFull.findings.map((f) => [f.ruleId, f.function, f.sink]));

  const assertions = [
    { name: 'clean version has no taint findings', pass: cleanCold.findings.length === 0 },
    { name: 'unrelated edit has no taint findings', pass: unrelatedIncremental.findings.length === 0 },
    { name: 'vulnerable edit produces at least one finding', pass: vulnerableIncremental.findings.length > 0 },
    { name: 'incremental and full vulnerable findings agree', pass: sameFindings },
    {
      name: 'incremental vulnerable analysis reuses most summaries',
      pass: vulnerableIncremental.metrics.recomputedFunctions < vulnerableFull.metrics.recomputedFunctions,
    },
  ];

  if (workload === 'layered') {
    assertions.push({
      name: 'layered workload propagates beyond the directly changed function',
      pass: vulnerableIncremental.metrics.recomputedFunctions > 2,
    });
    assertions.push({
      name: 'fan-out workload reports one finding per tainted branch',
      pass: vulnerableIncremental.findings.length === fanout,
    });
  }

  return assertions;
}

function runSuite(suite) {
  const suiteOutDir = path.join(outDir, suite.name);
  fs.mkdirSync(suiteOutDir, { recursive: true });

  fs.writeFileSync(path.join(suiteOutDir, 'clean.rs'), suite.clean);
  fs.writeFileSync(path.join(suiteOutDir, 'vulnerable.rs'), suite.vulnerable);
  fs.writeFileSync(path.join(suiteOutDir, 'unrelated_edit.rs'), suite.unrelatedEdit);

  const cleanCold = analyzeProject(suite.clean);
  const unrelatedIncremental = analyzeProject(suite.unrelatedEdit, cleanCold);
  const vulnerableIncremental = analyzeProject(suite.vulnerable, cleanCold);
  const vulnerableFull = analyzeProject(suite.vulnerable);

  const runResults = [
    { name: 'clean cold', result: cleanCold },
    { name: 'unrelated incremental', result: unrelatedIncremental },
    { name: 'vulnerable incremental', result: vulnerableIncremental },
    { name: 'vulnerable full', result: vulnerableFull },
  ];
  const resultsByName = new Map(runResults.map((r) => [r.name, r.result]));
  const assertions = buildAssertions(resultsByName, suite.name);

  const resultPayload = {
    workload: suite.name,
    description: suite.description,
    scale,
    chainDepth: suite.name === 'layered' ? chainDepth : undefined,
    fanout: suite.name === 'layered' ? fanout : undefined,
    assertions,
    scenarios: runResults.map((r) => ({
      name: r.name,
      metrics: r.result.metrics,
      findings: r.result.findings,
    })),
  };

  writeJsonAt(suiteOutDir, 'results.json', resultPayload);
  writeJsonAt(
    suiteOutDir,
    'diagnostics.json',
    diagnosticsFor(`file:///workspace/${suite.name}/vulnerable.rs`, vulnerableIncremental.findings),
  );
  writeJsonAt(suiteOutDir, 'semantic-facts.json', semanticFacts(vulnerableIncremental));

  printSummary(suite.name, runResults, assertions);
  return resultPayload;
}

function buildSynAssertions(runResults, edited) {
  const resultsByName = new Map(runResults.map((r) => [r.name, r.result]));
  const cold = resultsByName.get('syn cold');
  const assertions = [
    { name: 'syn fact file contains functions', pass: cold.metrics.functions > 0 },
    {
      name: 'syn fact file exposes source or sink hints',
      pass: cold.metrics.sourceHints + cold.metrics.sinkHints > 0,
    },
  ];

  if (edited) {
    const editIncremental = resultsByName.get('syn edit incremental');
    const editFull = resultsByName.get('syn edit full');
    const sameFindings =
      JSON.stringify(editIncremental.findings.map((f) => [f.ruleId, f.function, f.sink])) ===
      JSON.stringify(editFull.findings.map((f) => [f.ruleId, f.function, f.sink]));

    assertions.push(
      { name: 'syn edit changes at least one function hash', pass: editIncremental.metrics.changedFunctions > 0 },
      { name: 'syn incremental and full findings agree', pass: sameFindings },
      {
        name: 'syn incremental analysis reuses summaries after edit',
        pass: editIncremental.metrics.recomputedFunctions < editFull.metrics.recomputedFunctions,
      },
    );
  }

  return assertions;
}

function runSynFactsSuite() {
  const suiteOutDir = path.join(outDir, 'syn', synName);
  fs.mkdirSync(suiteOutDir, { recursive: true });

  const cleanFacts = readJson(synFactsPath);
  const editedFacts = synEditedFactsPath ? readJson(synEditedFactsPath) : null;
  const cleanCold = analyzeFactProject(cleanFacts);
  const runResults = [{ name: 'syn cold', result: cleanCold }];

  if (editedFacts) {
    runResults.push(
      { name: 'syn edit incremental', result: analyzeFactProject(editedFacts, cleanCold) },
      { name: 'syn edit full', result: analyzeFactProject(editedFacts) },
    );
  }

  const assertions = buildSynAssertions(runResults, Boolean(editedFacts));
  const primaryResult = editedFacts ? runResults[1].result : cleanCold;
  const source = editedFacts?.source ?? cleanFacts.source ?? path.resolve(synFactsPath);
  const uri = pathToFileURL(path.isAbsolute(source) ? source : path.resolve(source)).href;

  const resultPayload = {
    workload: `syn/${synName}`,
    description: 'Summary invalidation over syn-sidecar function facts from a real Rust source file.',
    source,
    schema: cleanFacts.schema,
    assertions,
    scenarios: runResults.map((r) => ({
      name: r.name,
      metrics: r.result.metrics,
      findings: r.result.findings,
    })),
  };

  writeJsonAt(suiteOutDir, 'results.json', resultPayload);
  writeJsonAt(suiteOutDir, 'diagnostics.json', diagnosticsFor(uri, primaryResult.findings));
  writeJsonAt(
    suiteOutDir,
    'semantic-facts.json',
    semanticFacts(primaryResult, {
      kind: 'syn-sidecar-adapter',
      parser: 'syn',
      replaceableBy: ['rust-analyzer-hir'],
    }),
  );

  printSummary(`syn/${synName}`, runResults, assertions);
  return resultPayload;
}

function buildSafAssertions(runResults, edited, facts, editedFacts) {
  const resultsByName = new Map(runResults.map((r) => [r.name, r.result]));
  const cold = resultsByName.get('SAF cold');
  const assertions = [
    { name: 'SAF fact file contains functions', pass: cold.metrics.functions > 0 },
    {
      name: 'SAF direct findings agree with LiveSafeRust diagnostics',
      pass: cold.findings.length === (facts.saf?.directFindingCount ?? facts.findings?.length ?? 0),
    },
  ];

  if ((facts.saf?.directFindingCount ?? facts.findings?.length ?? 0) > 0) {
    assertions.push({
      name: 'SAF produces at least one finding',
      pass: cold.findings.length > 0,
    });
  }

  if (edited) {
    const editIncremental = resultsByName.get('SAF edit incremental');
    const editFull = resultsByName.get('SAF edit full');
    const sameFindings =
      JSON.stringify(editIncremental.findings.map((f) => [f.ruleId, f.function, f.sink])) ===
      JSON.stringify(editFull.findings.map((f) => [f.ruleId, f.function, f.sink]));

    assertions.push(
      { name: 'SAF edit changes at least one function hash or hits LLVM cache', pass: editIncremental.metrics.changedFunctions > 0 || editedFacts.saf?.cacheHit === true },
      { name: 'SAF incremental and full findings agree', pass: sameFindings },
      {
        name: 'SAF incremental orchestration reuses summaries after edit',
        pass: editIncremental.metrics.recomputedFunctions <= editFull.metrics.recomputedFunctions,
      },
    );

    if (editedFacts.saf?.cacheHit === true) {
      assertions.push({
        name: 'unchanged LLVM IR edit does not re-invoke SAF',
        pass: editedFacts.saf?.invoked === false,
      });
    }
  }

  return assertions;
}

function runSafBackedSuite() {
  if (!safFactsPath) {
    throw new Error('--workload saf-backed requires --facts <saf-output.json>');
  }

  const suiteOutDir = path.join(outDir, 'saf-backed', safName);
  fs.mkdirSync(suiteOutDir, { recursive: true });

  const cleanFacts = readJson(safFactsPath);
  const editedFacts = safEditedFactsPath ? readJson(safEditedFactsPath) : null;
  const cleanCold = analyzeSafProject(cleanFacts);
  const runResults = [{ name: 'SAF cold', result: cleanCold }];

  if (editedFacts) {
    runResults.push(
      { name: 'SAF edit incremental', result: analyzeSafProject(editedFacts, cleanCold) },
      { name: 'SAF edit full', result: analyzeSafProject(editedFacts) },
    );
  }

  const assertions = buildSafAssertions(runResults, Boolean(editedFacts), cleanFacts, editedFacts);
  const primaryResult = editedFacts ? runResults[1].result : cleanCold;
  const source = editedFacts?.input?.source ?? cleanFacts.input?.source ?? cleanFacts.input?.llvmIr ?? path.resolve(safFactsPath);
  const uri = pathToFileURL(path.isAbsolute(source) ? source : path.resolve(source)).href;

  const resultPayload = {
    workload: `saf-backed/${safName}`,
    description: 'LiveSafeRust orchestration over SAF Python SDK findings and per-function LLVM facts.',
    source,
    schema: cleanFacts.schema,
    saf: {
      cold: cleanFacts.saf,
      edit: editedFacts?.saf,
    },
    timings: {
      cold: cleanFacts.timings,
      edit: editedFacts?.timings,
    },
    assertions,
    scenarios: runResults.map((r) => ({
      name: r.name,
      metrics: r.result.metrics,
      findings: r.result.findings,
    })),
  };

  writeJsonAt(suiteOutDir, 'results.json', resultPayload);
  writeJsonAt(suiteOutDir, 'diagnostics.json', diagnosticsFor(uri, primaryResult.findings));
  writeJsonAt(
    suiteOutDir,
    'semantic-facts.json',
    semanticFacts(primaryResult, {
      kind: 'saf-backed',
      parser: cleanFacts.frontend?.parser ?? 'LLVM IR -> SAF',
      replaceableBy: ['rust-analyzer-hir-to-SAF-AIR'],
      saf: cleanFacts.saf,
    }),
  );

  printSummary(`saf-backed/${safName}`, runResults, assertions);
  return resultPayload;
}

let allResults;
if (workloadArg === 'saf-backed') {
  allResults = [runSafBackedSuite()];
} else if (synFactsPath) {
  allResults = [runSynFactsSuite()];
} else {
  const workloads = workloadArg === 'all' ? ['flat', 'layered'] : [workloadArg];
  allResults = workloads.map((workload) => runSuite(generateSuite(workload)));
}

writeJson('summary.json', {
  generatedAt: new Date().toISOString(),
  workloads: allResults.map((result) => ({
    workload: result.workload,
    assertions: result.assertions,
    scenarios: result.scenarios.map((scenario) => ({
      name: scenario.name,
      metrics: scenario.metrics,
      findingCount: scenario.findings.length,
    })),
  })),
});

const failed = allResults.flatMap((result) => result.assertions).some((assertion) => !assertion.pass);
console.log(`\nWrote ${outDir}`);
if (failed) {
  process.exitCode = 1;
}
