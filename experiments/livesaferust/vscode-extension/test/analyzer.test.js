const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { analyzeRustText } = require('../livesaferust-analyzer');

const root = path.resolve(__dirname, '..');

function analyzeSample(name) {
  const file = path.join(root, 'sample', name);
  return analyzeRustText(fs.readFileSync(file, 'utf8'), { fileName: file });
}

{
  const result = analyzeSample('vulnerable.rs');
  assert.equal(result.diagnostics.length, 1);
  assert.equal(result.diagnostics[0].code, 'tainted-command');
  assert.match(result.diagnostics[0].message, /Command::new/);
}

{
  const result = analyzeSample('sanitized.rs');
  assert.equal(result.diagnostics.length, 0);
}

{
  const result = analyzeSample('path_traversal.rs');
  assert.equal(result.diagnostics.length, 1);
  assert.equal(result.diagnostics[0].code, 'tainted-path');
}

console.log('LiveSafeRust VS Code analyzer tests passed');
