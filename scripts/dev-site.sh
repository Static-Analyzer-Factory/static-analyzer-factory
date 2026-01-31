#!/usr/bin/env bash
# Unified dev server with hot reload for the full site.
# Runs Vite dev servers for site/ and tutorials/, builds docs once,
# then proxies everything through a single origin on port 8080.
#
# Usage: bash scripts/dev-site.sh
#        make dev  (if wired in Makefile)

set -euo pipefail
trap 'kill 0 2>/dev/null; exit' INT TERM EXIT

PORT=${PORT:-8080}
SITE_PORT=5173
TUTORIALS_PORT=5174

# Kill any stale processes on our ports
for p in $PORT $SITE_PORT $TUTORIALS_PORT; do
  pid=$(lsof -ti:"$p" 2>/dev/null || true)
  if [ -n "$pid" ]; then
    echo "Killing stale process on port $p (pid $pid)"
    kill $pid 2>/dev/null || true
    sleep 0.5
  fi
done

echo "=== Building docs (one-time) ==="
cd "$(git rev-parse --show-toplevel)"
if [ -d docs/book ]; then
  mdbook build docs/book 2>/dev/null || echo "(mdbook not installed, skipping docs)"
fi

echo "=== Installing workspace dependencies ==="
npm install --silent

echo "=== Copying tree-sitter WASM to tutorials ==="
cp -n playground/public/tree-sitter-llvm.wasm playground/public/web-tree-sitter.wasm tutorials/public/ 2>/dev/null || true

echo "=== Starting site dev server on :$SITE_PORT ==="
(cd site && npx vite --port $SITE_PORT --strictPort) &

echo "=== Starting tutorials dev server on :$TUTORIALS_PORT ==="
(cd tutorials && DEV_BASE=/tutorials/ npx vite --port $TUTORIALS_PORT --strictPort) &

# Wait for dev servers to be ready
for p in $SITE_PORT $TUTORIALS_PORT; do
  for _ in $(seq 1 30); do
    if curl -s -o /dev/null "http://localhost:$p" 2>/dev/null; then break; fi
    sleep 0.5
  done
done

echo "=== Starting reverse proxy on :$PORT ==="
# Use a simple Node.js proxy (no extra deps — uses built-in http module)
node -e "
const http = require('http');
const { URL } = require('url');

const routes = [
  { prefix: '/tutorials', target: 'http://localhost:$TUTORIALS_PORT', rewrite: '' },
  { prefix: '/docs', target: null },       // static from docs/book/build
  { prefix: '/playground', target: null },  // not running in dev
  { prefix: '/', target: 'http://localhost:$SITE_PORT', rewrite: '' },
];

const fs = require('fs');
const path = require('path');
const root = '$(git rev-parse --show-toplevel)';

function serveStatic(res, filePath) {
  const ext = path.extname(filePath);
  const types = { '.html':'text/html', '.css':'text/css', '.js':'application/javascript', '.json':'application/json', '.png':'image/png', '.svg':'image/svg+xml' };
  try {
    let p = filePath;
    if (fs.existsSync(p) && fs.statSync(p).isDirectory()) p = path.join(p, 'index.html');
    if (!fs.existsSync(p) && !ext) p = filePath + '.html';
    if (!fs.existsSync(p)) { res.writeHead(404); res.end('Not found'); return; }
    res.writeHead(200, { 'Content-Type': types[path.extname(p)] || 'application/octet-stream' });
    fs.createReadStream(p).pipe(res);
  } catch { res.writeHead(500); res.end('Error'); }
}

const server = http.createServer((req, res) => {
  const url = req.url || '/';

  // Tutorials (pass path through — Vite serves at /tutorials/ base)
  if (url.startsWith('/tutorials')) {
    const opts = { hostname: 'localhost', port: $TUTORIALS_PORT, path: url, method: req.method, headers: { ...req.headers, host: 'localhost:$TUTORIALS_PORT' } };
    const proxy = http.request(opts, (proxyRes) => {
      // Rewrite WebSocket upgrade path for HMR
      res.writeHead(proxyRes.statusCode, proxyRes.headers);
      proxyRes.pipe(res);
    });
    proxy.on('error', () => { res.writeHead(502); res.end('tutorials dev server not ready'); });
    req.pipe(proxy);
    return;
  }

  // Playground (serve pre-built dist if available, otherwise show placeholder)
  if (url.startsWith('/playground')) {
    const distDir = path.join(root, 'playground/dist');
    if (fs.existsSync(distDir)) {
      const file = url.replace('/playground', '') || '/index.html';
      serveStatic(res, path.join(distDir, file));
    } else {
      res.writeHead(200, { 'Content-Type': 'text/html' });
      res.end(\`<!DOCTYPE html>
<html><head><title>Playground — Dev Mode</title>
<style>body{font-family:system-ui;display:flex;align-items:center;justify-content:center;min-height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0}
.card{text-align:center;padding:2rem;border:1px solid #333;border-radius:12px;max-width:480px}
h1{font-size:1.5rem;margin-bottom:1rem}code{background:#2a2a4a;padding:2px 8px;border-radius:4px;font-size:0.9rem}
a{color:#6c9fff;text-decoration:none}a:hover{text-decoration:underline}</style></head>
<body><div class="card"><h1>Playground not available in dev mode</h1>
<p>The playground requires a WASM build. Run <code>make playground</code> first, then reload.</p>
<p style="margin-top:1.5rem"><a href="/">← Back to Home</a></p></div></body></html>\`);
    }
    return;
  }

  // Docs (static)
  if (url.startsWith('/docs')) {
    const file = url.replace('/docs', '') || '/index.html';
    serveStatic(res, path.join(root, 'docs/book/build', file));
    return;
  }

  // Everything else -> site dev server
  const opts = { hostname: 'localhost', port: $SITE_PORT, path: url, method: req.method, headers: { ...req.headers, host: 'localhost:$SITE_PORT' } };
  const proxy = http.request(opts, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res);
  });
  proxy.on('error', () => { res.writeHead(502); res.end('site dev server not ready'); });
  req.pipe(proxy);
});

// WebSocket upgrade for Vite HMR
server.on('upgrade', (req, socket, head) => {
  const url = req.url || '/';
  let targetPort = $SITE_PORT;
  let newPath = url;
  if (url.startsWith('/tutorials')) {
    targetPort = $TUTORIALS_PORT;
    newPath = url;  // pass through — Vite serves at /tutorials/ base
  }
  const opts = { hostname: 'localhost', port: targetPort, path: newPath, method: 'GET', headers: { ...req.headers, host: 'localhost:' + targetPort } };
  const proxy = http.request(opts);
  proxy.on('upgrade', (proxyRes, proxySocket, proxyHead) => {
    socket.write('HTTP/1.1 101 Switching Protocols\r\n' +
      Object.entries(proxyRes.headers).map(([k,v]) => k + ': ' + v).join('\r\n') + '\r\n\r\n');
    if (proxyHead.length) socket.write(proxyHead);
    proxySocket.pipe(socket);
    socket.pipe(proxySocket);
  });
  proxy.on('error', () => socket.destroy());
  proxy.end();
});

server.listen($PORT, () => {
  const hasPlayground = fs.existsSync(path.join(root, 'playground/dist/index.html'));
  console.log('');
  console.log('  Dev site ready:');
  console.log('    Landing:    http://localhost:$PORT/');
  console.log('    Tutorials:  http://localhost:$PORT/tutorials/');
  console.log('    Playground: http://localhost:$PORT/playground/' + (hasPlayground ? '' : ' (not built — run make playground)'));
  console.log('    Docs:       http://localhost:$PORT/docs/');
  console.log('');
  console.log('  Hot reload active for site/ and tutorials/');
  console.log('  Press Ctrl+C to stop');
  console.log('');
});
"
