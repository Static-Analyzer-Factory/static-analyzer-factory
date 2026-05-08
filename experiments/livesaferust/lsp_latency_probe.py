#!/usr/bin/env python3
"""Measure rust-analyzer didChange -> publishDiagnostics latency.

This is a tiny JSON-RPC/LSP client for the LiveSafeRust proposal. It opens a
Rust file, waits for optional initial diagnostics, sends an in-memory full-text
change, and records the time until rust-analyzer publishes diagnostics for the
same URI.

Example:
    python3 experiments/livesaferust/lsp_latency_probe.py \
      --crate /path/to/duct \
      --file src/lib.rs \
      --out experiments/livesaferust/out/baseline/ra-lsp-latency.json
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import queue
import subprocess
import sys
import threading
import time
from typing import Any, Dict, Optional
from urllib.parse import quote


Json = Dict[str, Any]


def file_uri(path: pathlib.Path) -> str:
    resolved = path.resolve()
    return "file://" + quote(str(resolved))


def read_message(stream) -> Optional[Json]:
    headers = {}
    while True:
        line = stream.readline()
        if not line:
            return None
        line = line.decode("ascii", errors="replace").strip()
        if line == "":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            headers[key.lower()] = value.strip()

    length = int(headers.get("content-length", "0"))
    if length <= 0:
        return None
    payload = stream.read(length)
    if not payload:
        return None
    return json.loads(payload.decode("utf-8"))


def write_message(stream, message: Json) -> None:
    payload = json.dumps(message, separators=(",", ":")).encode("utf-8")
    header = f"Content-Length: {len(payload)}\r\n\r\n".encode("ascii")
    stream.write(header + payload)
    stream.flush()


class LspProbe:
    def __init__(self, args: argparse.Namespace) -> None:
        self.args = args
        self.proc: Optional[subprocess.Popen] = None
        self.reader_thread: Optional[threading.Thread] = None
        self.stderr_thread: Optional[threading.Thread] = None
        self.messages: "queue.Queue[Json]" = queue.Queue()
        self.next_id = 1
        self.stderr_lines = []

    def start(self) -> None:
        self.proc = subprocess.Popen(
            [self.args.rust_analyzer],
            cwd=str(self.args.crate),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        assert self.proc.stdout is not None
        assert self.proc.stderr is not None

        def read_stdout() -> None:
            while True:
                msg = read_message(self.proc.stdout)
                if msg is None:
                    break
                self.messages.put(msg)

        def read_stderr() -> None:
            for raw in self.proc.stderr:
                text = raw.decode("utf-8", errors="replace").rstrip()
                if text:
                    self.stderr_lines.append(text)

        self.reader_thread = threading.Thread(target=read_stdout, daemon=True)
        self.stderr_thread = threading.Thread(target=read_stderr, daemon=True)
        self.reader_thread.start()
        self.stderr_thread.start()

    def stop(self) -> None:
        if self.proc is None:
            return
        try:
            shutdown_id = self.request("shutdown", None, timeout=5.0)
            self.wait_response(shutdown_id, timeout=5.0)
            self.notify("exit", None)
        except Exception:
            pass
        try:
            self.proc.wait(timeout=3.0)
        except subprocess.TimeoutExpired:
            self.proc.kill()

    def request(self, method: str, params: Any, timeout: float = 10.0) -> int:
        if self.proc is None or self.proc.stdin is None:
            raise RuntimeError("rust-analyzer is not running")
        request_id = self.next_id
        self.next_id += 1
        write_message(
            self.proc.stdin,
            {"jsonrpc": "2.0", "id": request_id, "method": method, "params": params},
        )
        return request_id

    def notify(self, method: str, params: Any) -> None:
        if self.proc is None or self.proc.stdin is None:
            raise RuntimeError("rust-analyzer is not running")
        write_message(
            self.proc.stdin,
            {"jsonrpc": "2.0", "method": method, "params": params},
        )

    def wait_response(self, request_id: int, timeout: float) -> Json:
        deadline = time.perf_counter() + timeout
        while time.perf_counter() < deadline:
            try:
                msg = self.messages.get(timeout=0.05)
            except queue.Empty:
                continue
            if msg.get("id") == request_id:
                return msg
        raise TimeoutError(f"timed out waiting for response id {request_id}")

    def drain_until_diagnostics(
        self,
        uri: str,
        timeout: float,
        min_version: Optional[int] = None,
        require_nonempty: bool = False,
    ) -> Optional[Json]:
        deadline = time.perf_counter() + timeout
        while time.perf_counter() < deadline:
            try:
                msg = self.messages.get(timeout=0.05)
            except queue.Empty:
                continue
            if msg.get("method") == "textDocument/publishDiagnostics":
                params = msg.get("params", {})
                if params.get("uri") == uri:
                    version = params.get("version")
                    if min_version is not None and version is not None and version < min_version:
                        continue
                    if require_nonempty and not params.get("diagnostics", []):
                        continue
                    return msg
        return None

    def drain_pending(self, duration: float = 0.2) -> int:
        drained = 0
        deadline = time.perf_counter() + duration
        while time.perf_counter() < deadline:
            try:
                self.messages.get(timeout=0.02)
                drained += 1
            except queue.Empty:
                pass
        return drained

    def initialize(self, root_uri: str) -> Json:
        init_id = self.request(
            "initialize",
            {
                "processId": os.getpid(),
                "rootUri": root_uri,
                "workspaceFolders": [
                    {"uri": root_uri, "name": pathlib.Path(self.args.crate).name}
                ],
                "capabilities": {
                    "textDocument": {
                        "publishDiagnostics": {"relatedInformation": True},
                        "synchronization": {
                            "didSave": True,
                            "dynamicRegistration": False,
                        },
                    },
                    "workspace": {"workspaceFolders": True},
                },
            },
        )
        response = self.wait_response(init_id, timeout=self.args.timeout)
        self.notify("initialized", {})
        return response


def changed_text(original: str, mode: str) -> str:
    marker = "// livesaferust-lsp-probe edit\n"
    if mode == "append-comment":
        if original.endswith("\n"):
            return original + marker
        return original + "\n" + marker
    if mode == "toggle-space":
        return original + " "
    if mode == "syntax-error":
        broken = "\nfn livesaferust_probe_broken( {\n"
        if original.endswith("\n"):
            return original + broken
        return original + "\n" + broken
    if mode == "semantic-error":
        broken = "\nfn livesaferust_probe_semantic_error() { does_not_exist_probe_symbol(); }\n"
        if original.endswith("\n"):
            return original + broken
        return original + "\n" + broken
    raise ValueError(f"unknown change mode: {mode}")


def run(args: argparse.Namespace) -> Json:
    crate = pathlib.Path(args.crate).resolve()
    file_path = (crate / args.file).resolve()
    if not file_path.exists():
        raise FileNotFoundError(file_path)

    uri = file_uri(file_path)
    root_uri = file_uri(crate)
    original = file_path.read_text(encoding="utf-8")
    changed = changed_text(original, args.change_mode)

    probe = LspProbe(args)
    started = time.perf_counter()
    probe.start()
    try:
        init_response = probe.initialize(root_uri)
        opened_at = time.perf_counter()
        probe.notify(
            "textDocument/didOpen",
            {
                "textDocument": {
                    "uri": uri,
                    "languageId": "rust",
                    "version": 1,
                    "text": original,
                }
            },
        )

        initial_diag = None
        if args.wait_initial:
            initial_diag = probe.drain_until_diagnostics(uri, timeout=args.timeout)
        drained_before_change = probe.drain_pending(args.quiesce_ms / 1000.0)

        change_sent_at = time.perf_counter()
        probe.notify(
            "textDocument/didChange",
            {
                "textDocument": {"uri": uri, "version": 2},
                "contentChanges": [{"text": changed}],
            },
        )
        if args.did_save:
            probe.notify("textDocument/didSave", {"textDocument": {"uri": uri}, "text": changed})
        changed_diag = probe.drain_until_diagnostics(
            uri,
            timeout=args.timeout,
            min_version=2,
            require_nonempty=args.require_nonempty,
        )
        received_at = time.perf_counter()

        return {
            "crate": str(crate),
            "file": str(file_path),
            "uri": uri,
            "rustAnalyzer": args.rust_analyzer,
            "changeMode": args.change_mode,
            "initialized": "result" in init_response,
            "initialDiagnosticsReceived": initial_diag is not None,
            "drainedBeforeChange": drained_before_change,
            "changedDiagnosticsReceived": changed_diag is not None,
            "processStartupMs": round((opened_at - started) * 1000, 3),
            "didChangeToDiagnosticsMs": (
                round((received_at - change_sent_at) * 1000, 3)
                if changed_diag is not None
                else None
            ),
            "changedDiagnosticCount": (
                len(changed_diag.get("params", {}).get("diagnostics", []))
                if changed_diag is not None
                else None
            ),
            "stderrTail": probe.stderr_lines[-20:],
        }
    finally:
        probe.stop()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--crate", type=pathlib.Path, required=True)
    parser.add_argument("--file", default="src/lib.rs")
    parser.add_argument("--rust-analyzer", default="rust-analyzer")
    parser.add_argument("--timeout", type=float, default=30.0)
    parser.add_argument(
        "--change-mode",
        choices=["append-comment", "toggle-space", "syntax-error", "semantic-error"],
        default="append-comment",
    )
    parser.add_argument("--wait-initial", action="store_true")
    parser.add_argument("--require-nonempty", action="store_true")
    parser.add_argument("--did-save", action="store_true")
    parser.add_argument("--quiesce-ms", type=float, default=200.0)
    parser.add_argument("--out", type=pathlib.Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    result = run(args)
    text = json.dumps(result, indent=2)
    if args.out:
        args.out.parent.mkdir(parents=True, exist_ok=True)
        args.out.write_text(text + "\n", encoding="utf-8")
    print(text)
    return 0 if result["changedDiagnosticsReceived"] else 1


if __name__ == "__main__":
    sys.exit(main())
