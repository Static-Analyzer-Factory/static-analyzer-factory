const SOURCE_DEP = '__source__';

const SINKS = [
  { name: 'Command::new', regex: /Command\s*::\s*new\s*\(([^)]*)\)/ },
  { name: 'Command::arg', regex: /\.arg\s*\(([^)]*)\)/ },
  { name: 'File::open', regex: /File\s*::\s*open\s*\(([^)]*)\)/ },
  { name: 'File::create', regex: /File\s*::\s*create\s*\(([^)]*)\)/ },
  { name: 'system', regex: /\bsystem\s*\(([^)]*)\)/ },
];

function parseParams(paramsText) {
  if (!paramsText.trim()) {
    return [];
  }
  return paramsText
    .split(',')
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => part.split(':')[0].trim().replace(/^mut\s+/, ''))
    .filter((name) => name && name !== 'self' && name !== '&self' && name !== '&mut self');
}

function countChar(text, ch) {
  return (text.match(new RegExp(`\\${ch}`, 'g')) || []).length;
}

function parseFunctions(text) {
  const lines = text.split(/\r?\n/);
  const functions = [];

  for (let i = 0; i < lines.length; i += 1) {
    const header = lines[i].match(/^\s*(?:pub\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(([^)]*)\)/);
    if (!header) {
      continue;
    }

    const name = header[1];
    const params = parseParams(header[2]);
    const startLine = i;
    let depth = countChar(lines[i], '{') - countChar(lines[i], '}');
    const body = [{ text: lines[i], line: i }];

    while (depth > 0 && i + 1 < lines.length) {
      i += 1;
      body.push({ text: lines[i], line: i });
      depth += countChar(lines[i], '{') - countChar(lines[i], '}');
    }

    functions.push({
      name,
      params,
      body,
      startLine,
      endLine: i,
    });
  }

  return functions;
}

function stableSet(values) {
  return [...new Set(values)].sort();
}

function setsEqual(a, b) {
  if (a.length !== b.length) {
    return false;
  }
  return a.every((value, index) => value === b[index]);
}

function union(...sets) {
  return stableSet(sets.flat());
}

function isSanitizerName(name) {
  return /(^|_)(sanitize|clean|escape|validate|canonicalize|normalize_safe)(_|$)/i.test(name);
}

function hasSourceExpression(expr) {
  return /\benv\s*::\s*(args|var)\s*\(/.test(expr)
    || /\bstd\s*::\s*env\s*::\s*(args|var)\s*\(/.test(expr)
    || /\bgetenv\s*\(/.test(expr);
}

function splitArgs(argsText) {
  const args = [];
  let current = '';
  let depth = 0;
  for (const ch of argsText) {
    if (ch === '(' || ch === '[' || ch === '{') depth += 1;
    if (ch === ')' || ch === ']' || ch === '}') depth -= 1;
    if (ch === ',' && depth === 0) {
      args.push(current.trim());
      current = '';
    } else {
      current += ch;
    }
  }
  if (current.trim()) {
    args.push(current.trim());
  }
  return args;
}

function callMatches(expr, functionNames) {
  const matches = [];
  for (const name of functionNames) {
    const regex = new RegExp(`\\b${name}\\s*\\(([^)]*)\\)`, 'g');
    let match;
    while ((match = regex.exec(expr)) !== null) {
      matches.push({ name, args: splitArgs(match[1]) });
    }
  }
  return matches;
}

function depsForExpression(expr, env, fn, summaries, functionNames) {
  if (/\bsanitize[A-Za-z0-9_]*\s*\(/.test(expr) || /\bcanonicalize\s*\(/.test(expr)) {
    return [];
  }

  let deps = [];
  if (hasSourceExpression(expr)) {
    deps.push(SOURCE_DEP);
  }

  for (const param of fn.params) {
    const index = fn.params.indexOf(param);
    const regex = new RegExp(`\\b${param}\\b`);
    if (regex.test(expr)) {
      deps.push(`param:${index}`);
    }
  }

  for (const [name, valueDeps] of env.entries()) {
    const regex = new RegExp(`\\b${name}\\b`);
    if (regex.test(expr)) {
      deps = union(deps, valueDeps);
    }
  }

  for (const call of callMatches(expr, functionNames)) {
    if (isSanitizerName(call.name)) {
      continue;
    }
    const summary = summaries.get(call.name) || { returnDeps: [], sinkParamDeps: [] };
    let callDeps = [];
    for (const dep of summary.returnDeps) {
      if (dep === SOURCE_DEP) {
        callDeps.push(SOURCE_DEP);
      } else if (dep.startsWith('param:')) {
        const index = Number.parseInt(dep.slice('param:'.length), 10);
        if (Number.isInteger(index) && call.args[index]) {
          callDeps = union(callDeps, depsForExpression(call.args[index], env, fn, summaries, functionNames));
        }
      }
    }
    deps = union(deps, callDeps);
  }

  return stableSet(deps);
}

function sinkArgument(line) {
  for (const sink of SINKS) {
    const match = line.match(sink.regex);
    if (match) {
      return { sink: sink.name, expression: match[1] || line };
    }
  }
  return null;
}

function returnExpression(fn) {
  for (let i = fn.body.length - 1; i >= 0; i -= 1) {
    const line = fn.body[i].text.trim();
    const ret = line.match(/^return\s+(.+?);?$/);
    if (ret) {
      return ret[1];
    }
    if (line && !line.startsWith('}') && !line.endsWith(';') && !line.startsWith('fn ')) {
      return line.replace(/,$/, '');
    }
  }
  return '';
}

function scanFunction(fn, summaries, functionNames, options = {}) {
  const env = new Map();
  const findings = [];
  let sinkParamDeps = [];

  for (const entry of fn.body) {
    const line = entry.text;
    const letMatch = line.match(/\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^=]*=\s*(.+?);?\s*$/);
    if (letMatch) {
      env.set(letMatch[1], depsForExpression(letMatch[2], env, fn, summaries, functionNames));
    }

    const assignment = line.match(/^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.+?);?\s*$/);
    if (assignment && env.has(assignment[1])) {
      env.set(assignment[1], depsForExpression(assignment[2], env, fn, summaries, functionNames));
    }

    const sink = sinkArgument(line);
    if (sink) {
      const deps = depsForExpression(sink.expression, env, fn, summaries, functionNames);
      sinkParamDeps = union(sinkParamDeps, deps.filter((dep) => dep.startsWith('param:')));
      if (deps.includes(SOURCE_DEP) || deps.some((dep) => dep.startsWith('param:'))) {
        const character = Math.max(0, line.indexOf(sink.sink.split('::').pop()));
        findings.push({
          code: sink.sink.startsWith('File::') ? 'tainted-path' : 'tainted-command',
          source: 'LiveSafeRust',
          message: `Possible tainted Rust input reaches ${sink.sink}`,
          range: {
            start: { line: entry.line, character },
            end: { line: entry.line, character: Math.min(line.length, character + sink.sink.length) },
          },
          function: fn.name,
          sink: sink.sink,
          trace: ['source', fn.name, sink.sink],
          fileName: options.fileName,
        });
      }
    }

    for (const call of callMatches(line, functionNames)) {
      const summary = summaries.get(call.name) || { returnDeps: [], sinkParamDeps: [] };
      for (const dep of summary.sinkParamDeps) {
        const index = Number.parseInt(dep.slice('param:'.length), 10);
        if (Number.isInteger(index) && call.args[index]) {
          const argDeps = depsForExpression(call.args[index], env, fn, summaries, functionNames);
          if (argDeps.includes(SOURCE_DEP) || argDeps.some((item) => item.startsWith('param:'))) {
            findings.push({
              code: 'tainted-command',
              source: 'LiveSafeRust',
              message: `Possible tainted Rust input reaches a sink through ${call.name}`,
              range: {
                start: { line: entry.line, character: Math.max(0, line.indexOf(call.name)) },
                end: { line: entry.line, character: Math.min(line.length, Math.max(0, line.indexOf(call.name)) + call.name.length) },
              },
              function: fn.name,
              sink: call.name,
              trace: ['source', fn.name, call.name],
              fileName: options.fileName,
            });
          }
        }
      }
    }
  }

  const returnDeps = isSanitizerName(fn.name)
    ? []
    : depsForExpression(returnExpression(fn), env, fn, summaries, functionNames);

  return {
    summary: {
      returnDeps: stableSet(returnDeps),
      sinkParamDeps: stableSet(sinkParamDeps),
    },
    findings,
  };
}

function dedupeDiagnostics(diagnostics) {
  const seen = new Set();
  return diagnostics.filter((diagnostic) => {
    const key = `${diagnostic.code}:${diagnostic.range.start.line}:${diagnostic.sink}:${diagnostic.function}`;
    if (seen.has(key)) {
      return false;
    }
    seen.add(key);
    return true;
  });
}

function analyzeRustText(text, options = {}) {
  const functions = parseFunctions(text);
  const functionNames = functions.map((fn) => fn.name);
  const summaries = new Map(functions.map((fn) => [fn.name, { returnDeps: [], sinkParamDeps: [] }]));

  for (let iter = 0; iter < 12; iter += 1) {
    let changed = false;
    for (const fn of functions) {
      const next = scanFunction(fn, summaries, functionNames, options).summary;
      const prev = summaries.get(fn.name);
      if (!setsEqual(prev.returnDeps, next.returnDeps) || !setsEqual(prev.sinkParamDeps, next.sinkParamDeps)) {
        summaries.set(fn.name, next);
        changed = true;
      }
    }
    if (!changed) {
      break;
    }
  }

  const diagnostics = [];
  for (const fn of functions) {
    diagnostics.push(...scanFunction(fn, summaries, functionNames, options).findings);
  }

  return {
    schema: 'livesaferust.vscode-source-sidecar/0.1',
    uri: options.uri,
    diagnostics: dedupeDiagnostics(diagnostics),
    functions: functions.map((fn) => ({
      name: fn.name,
      startLine: fn.startLine,
      endLine: fn.endLine,
      summary: summaries.get(fn.name),
    })),
  };
}

module.exports = {
  analyzeRustText,
  parseFunctions,
};
