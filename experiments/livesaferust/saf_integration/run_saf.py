#!/usr/bin/env python3
"""Run SAF on Rust/C source and emit LiveSafeRust-compatible facts.

Host mode compiles source to LLVM IR, invokes the SAF Python SDK inside the
repository Docker image, and writes a JSON fact file. The `--inside` mode is an
implementation detail used inside the container.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[3]
SCRIPT = Path(__file__).resolve()
DEFAULT_SOURCES = ["env::args", "env::var", "getenv", "*env*args*", "*env*var*"]
DEFAULT_SINKS = [
    "Command::new",
    "*Command*new*",
    "Command::arg",
    "*Command*arg*",
    "File::open",
    "*File*open*",
    "File::create",
    "*File*create*",
    "system",
    "execve",
]
DEFAULT_SANITIZERS = ["sanitize_cmd", "sanitize_input", "sanitize_path", "*sanitize*"]
DIAGNOSTIC_UNDERLINE_WIDTH = 8
SAF_SERVICES = {
    "llvm18": "dev",
    "llvm22": "dev-llvm22",
}


def now_ms() -> float:
    """Return monotonic time in milliseconds."""

    return time.perf_counter() * 1000.0


def sha256_file(path: Path) -> str:
    """Hash a file."""

    h = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_text(text: str) -> str:
    """Hash text."""

    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def canonical_semantic_llvm_lines(lines: list[str]) -> list[str]:
    """Return LLVM IR lines with debug noise and local SSA names normalized."""

    kept = []
    local_names: dict[str, str] = {}

    def canonicalize_local_names(line: str) -> str:
        nonlocal local_names
        if line.lstrip().startswith("define "):
            local_names = {}

        def replace(match: re.Match[str]) -> str:
            name = match.group(0)
            if name not in local_names:
                local_names[name] = f"%v{len(local_names)}"
            return local_names[name]

        return re.sub(r"%[A-Za-z_.$][A-Za-z0-9_.$]*", replace, line)

    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith(";"):
            continue
        if stripped.startswith("source_filename"):
            continue
        if stripped.startswith("!") or re.match(r"^!\d+\s*=", stripped):
            continue
        line = re.sub(r",\s*!(dbg|prof|srcloc)\s+!\d+", "", line)
        kept.append(canonicalize_local_names(line))
    return kept


def semantic_llvm_sha256(path: Path) -> str:
    """Hash LLVM IR after removing debug noise and local SSA alpha-renames."""

    return sha256_text(
        "\n".join(canonical_semantic_llvm_lines(path.read_text(errors="replace").splitlines()))
    )


def semantic_llvm_body_hash(body_text: str) -> str:
    """Hash one LLVM function body with the same cache-stable normalization."""

    return sha256_text("\n".join(canonical_semantic_llvm_lines(body_text.splitlines())))[:16]


def metadata_string(body: str, key: str) -> str | None:
    """Extract a quoted string field from one LLVM debug metadata body."""

    match = re.search(rf'\b{re.escape(key)}:\s*"((?:\\.|[^"])*)"', body)
    if not match:
        return None
    return match.group(1).replace(r"\"", '"')


def metadata_ref(body: str, key: str) -> int | None:
    """Extract a metadata reference field like `file: !120`."""

    match = re.search(rf"\b{re.escape(key)}:\s*!(\d+)", body)
    return int(match.group(1)) if match else None


def metadata_int(body: str, key: str) -> int | None:
    """Extract an integer field from one LLVM debug metadata body."""

    match = re.search(rf"\b{re.escape(key)}:\s*(\d+)", body)
    return int(match.group(1)) if match else None


def normalize_metadata_path(filename: str | None, directory: str | None) -> str | None:
    """Turn LLVM DIFile filename/directory fields into a comparable path."""

    if not filename:
        return None
    filename = filename.split("/@/", 1)[0]
    if os.path.isabs(filename):
        return str(Path(filename))
    if directory:
        return str(Path(directory) / filename)
    return filename


def llvm_file_metadata(text: str) -> dict[int, str]:
    """Parse LLVM `!DIFile` metadata."""

    files: dict[int, str] = {}
    for match in re.finditer(r"^!(\d+)\s*=\s*!DIFile\((.*?)\)$", text, flags=re.MULTILINE):
        body = match.group(2)
        path = normalize_metadata_path(metadata_string(body, "filename"), metadata_string(body, "directory"))
        if path:
            files[int(match.group(1))] = path
    return files


def llvm_subprogram_metadata(text: str) -> dict[int, dict[str, Any]]:
    """Parse LLVM `!DISubprogram` metadata relevant to source ownership."""

    files = llvm_file_metadata(text)
    subprograms: dict[int, dict[str, Any]] = {}
    for match in re.finditer(
        r"^!(\d+)\s*=\s*(?:distinct\s+)?!DISubprogram\((.*?)\)$",
        text,
        flags=re.MULTILINE,
    ):
        body = match.group(2)
        file_ref = metadata_ref(body, "file")
        subprograms[int(match.group(1))] = {
            "file": files.get(file_ref) if file_ref is not None else None,
            "line": metadata_int(body, "line"),
            "name": metadata_string(body, "name"),
            "linkageName": metadata_string(body, "linkageName"),
        }
    return subprograms


def llvm_scope_files(text: str) -> dict[int, str]:
    """Map debug scopes to source files."""

    files = llvm_file_metadata(text)
    scope_files: dict[int, str] = {}
    parents: dict[int, int] = {}

    for scope_id, info in llvm_subprogram_metadata(text).items():
        if info.get("file"):
            scope_files[scope_id] = str(info["file"])

    for match in re.finditer(
        r"^!(\d+)\s*=\s*(?:distinct\s+)?!DILexicalBlock(?:File)?\((.*?)\)$",
        text,
        flags=re.MULTILINE,
    ):
        scope_id = int(match.group(1))
        body = match.group(2)
        file_ref = metadata_ref(body, "file")
        parent_ref = metadata_ref(body, "scope")
        if file_ref is not None and file_ref in files:
            scope_files[scope_id] = files[file_ref]
        if parent_ref is not None:
            parents[scope_id] = parent_ref

    def resolve(scope_id: int, seen: set[int] | None = None) -> str | None:
        if scope_id in scope_files:
            return scope_files[scope_id]
        seen = seen or set()
        if scope_id in seen:
            return None
        seen.add(scope_id)
        parent = parents.get(scope_id)
        if parent is None:
            return None
        resolved = resolve(parent, seen)
        if resolved:
            scope_files[scope_id] = resolved
        return resolved

    for scope_id in list(parents):
        resolve(scope_id)
    return scope_files


def llvm_debug_location_records(text: str) -> dict[int, dict[str, Any]]:
    """Parse LLVM `!DILocation` metadata into source location records."""

    scope_files = llvm_scope_files(text)
    records: dict[int, dict[str, Any]] = {}
    for match in re.finditer(
        r"^!(\d+)\s*=\s*(?:distinct\s+)?!DILocation\((.*?)\)$",
        text,
        flags=re.MULTILINE,
    ):
        body = match.group(2)
        line = metadata_int(body, "line")
        if not line:
            continue
        column = metadata_int(body, "column") or 1
        scope = metadata_ref(body, "scope")
        records[int(match.group(1))] = {
            "line": line,
            "column": max(1, column),
            "scope": scope,
            "file": scope_files.get(scope) if scope is not None else None,
        }
    return records


def paths_match(left: str | None, right: str | None) -> bool:
    """Compare host/container/relative paths produced by LLVM and SAF."""

    if not left or not right:
        return False
    left_norm = str(Path(left))
    right_norm = str(Path(right))
    if left_norm == right_norm:
        return True
    return left_norm.endswith(right_norm) or right_norm.endswith(left_norm)


def function_source_range(
    body_text: str,
    dbg_id: int | None,
    debug_locations: dict[int, dict[str, Any]],
    scope_files: dict[int, str],
    subprograms: dict[int, dict[str, Any]],
) -> dict[str, Any] | None:
    """Compute a function's source range from LLVM debug metadata."""

    function_file = scope_files.get(dbg_id) if dbg_id is not None else None
    lines: list[int] = []
    for match in re.finditer(r"!\bdbg\s+!(\d+)", body_text):
        record = debug_locations.get(int(match.group(1)))
        if not record:
            continue
        if function_file and record.get("file") and not paths_match(function_file, str(record["file"])):
            continue
        lines.append(int(record["line"]))

    if lines and function_file:
        return {
            "file": function_file,
            "startLine": min(lines),
            "endLine": max(lines),
        }

    subprogram = subprograms.get(dbg_id) if dbg_id is not None else None
    if subprogram and subprogram.get("file") and subprogram.get("line"):
        line = int(subprogram["line"])
        return {
            "file": subprogram["file"],
            "startLine": line,
            "endLine": line,
        }
    return None


def run_command(cmd: list[str], *, cwd: Path = ROOT) -> tuple[int, str, str, float]:
    """Run a command and capture output."""

    start = now_ms()
    proc = subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    return proc.returncode, proc.stdout, proc.stderr, now_ms() - start


def repo_relative(path: Path) -> str:
    """Return a path relative to the repo when possible."""

    try:
        return str(path.resolve().relative_to(ROOT))
    except ValueError:
        return str(path.resolve())


def to_container_path(path: Path) -> str:
    """Convert a repo path to its `/workspace` container path."""

    resolved = path.resolve()
    rel = resolved.relative_to(ROOT)
    return f"/workspace/{rel.as_posix()}"


def parse_llvm_functions(llvm_path: Path) -> list[dict[str, Any]]:
    """Extract lightweight per-function facts from LLVM text IR."""

    text = llvm_path.read_text(errors="replace")
    lines = text.splitlines()
    debug_locations = llvm_debug_location_records(text)
    scope_files = llvm_scope_files(text)
    subprograms = llvm_subprogram_metadata(text)
    functions: list[dict[str, Any]] = []
    pending_display: str | None = None
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        comment_match = re.match(r";\s*(.+)$", stripped)
        if comment_match:
            pending_display = comment_match.group(1).strip()

        define_match = re.match(r'define\b.*@(?:"([^"]+)"|([^\s(]+))\(', stripped)
        declare_match = re.match(r'declare\b.*@(?:"([^"]+)"|([^\s(]+))\(', stripped)
        if define_match or declare_match:
            match = define_match or declare_match
            assert match is not None
            name = match.group(1) or match.group(2)
            dbg_match = re.search(r"!\bdbg\s+!(\d+)", stripped)
            dbg_id = int(dbg_match.group(1)) if dbg_match else None
            is_decl = declare_match is not None
            start = i
            body: list[str] = [lines[i]]
            if not is_decl:
                depth = lines[i].count("{") - lines[i].count("}")
                i += 1
                while i < len(lines) and depth > 0:
                    body.append(lines[i])
                    depth += lines[i].count("{") - lines[i].count("}")
                    i += 1
                i -= 1
            body_text = "\n".join(body)
            calls = sorted(
                set(
                    m.group(1) or m.group(2)
                    for m in re.finditer(
                        r'\b(?:call|invoke)\b[^@]*@(?:"([^"]+)"|([^\s(]+))\(',
                        body_text,
                    )
                )
            )
            source_range = function_source_range(body_text, dbg_id, debug_locations, scope_files, subprograms)
            functions.append(
                {
                    "name": name,
                    "displayName": pending_display if pending_display and not pending_display.startswith("Function Attrs") else name,
                    "params": [],
                    "range": {"startLine": start + 1, "endLine": i + 1},
                    "sourceRange": source_range,
                    "hash": semantic_llvm_body_hash(body_text),
                    "calls": calls,
                    "isDeclaration": is_decl,
                    "summary": {"returnDeps": [], "sinkParams": []},
                    "findings": [],
                }
            )
            pending_display = None
        i += 1
    return functions


def rustup_command(toolchain: str | None, program: str) -> list[str]:
    """Build a command routed through rustup when a toolchain is requested."""

    if toolchain:
        rustup = os.environ.get("RUSTUP", str(Path.home() / ".cargo/bin/rustup"))
        return [rustup, "run", toolchain, program]
    if program == "rustc":
        return [os.environ.get("RUSTC", str(Path.home() / ".cargo/bin/rustc"))]
    if program == "cargo":
        return [os.environ.get("CARGO", str(Path.home() / ".cargo/bin/cargo"))]
    return [program]


def compile_rust(
    source: Path,
    llvm_path: Path,
    *,
    toolchain: str | None,
    crate_name: str | None,
) -> dict[str, Any]:
    """Compile a standalone Rust source file to LLVM IR."""

    llvm_path.parent.mkdir(parents=True, exist_ok=True)
    cmd = [
        *rustup_command(toolchain, "rustc"),
        "--edition=2021",
        "--emit=llvm-ir",
        "-C",
        "debuginfo=1",
    ]
    if crate_name:
        cmd.extend(["--crate-name", crate_name])
    cmd.extend([str(source), "-o", str(llvm_path)])
    code, stdout, stderr, elapsed = run_command(cmd)
    return {
        "kind": "rustc",
        "toolchain": toolchain or "host-default",
        "crateName": crate_name,
        "command": cmd,
        "exitCode": code,
        "stdout": stdout,
        "stderr": stderr,
        "elapsedMs": elapsed,
    }


def compile_rust_in_docker(
    source: Path,
    llvm_path: Path,
    *,
    toolchain: str,
    saf_image: str,
    crate_name: str | None,
) -> dict[str, Any]:
    """Compile a Rust source file to LLVM IR inside the SAF Docker image."""

    llvm_path.parent.mkdir(parents=True, exist_ok=True)
    service = SAF_SERVICES[saf_image]
    cmd = (
        f"mkdir -p {shlex.quote(to_container_path(llvm_path.parent))} && "
        f"rustup toolchain install {shlex.quote(toolchain)} >/tmp/livesaferust-rustup.log 2>&1 && "
        f"rustup run {shlex.quote(toolchain)} rustc --edition=2021 --emit=llvm-ir "
        f"-C debuginfo=1 "
    )
    if crate_name:
        cmd += f"--crate-name {shlex.quote(crate_name)} "
    cmd += (
        f"{shlex.quote(to_container_path(source))} "
        f"-o {shlex.quote(to_container_path(llvm_path))}"
    )
    docker_cmd = ["docker", "compose", "run", "--rm", service, "sh", "-c", cmd]
    code, stdout, stderr, elapsed = run_command(docker_cmd)
    return {
        "kind": "rustc-docker",
        "toolchain": toolchain,
        "crateName": crate_name,
        "safImage": saf_image,
        "command": docker_cmd,
        "exitCode": code,
        "stdout": stdout,
        "stderr": stderr,
        "elapsedMs": elapsed,
    }


def compile_c_in_docker(
    source: Path,
    llvm_path: Path,
    *,
    line_tables: bool,
    saf_image: str,
) -> dict[str, Any]:
    """Compile C source to LLVM IR using clang-18 inside Docker."""

    llvm_path.parent.mkdir(parents=True, exist_ok=True)
    debug_flag = "-gline-tables-only" if line_tables else "-g0"
    llvm_version = "22" if saf_image == "llvm22" else "18"
    service = SAF_SERVICES[saf_image]
    cmd = (
        f"clang-{llvm_version} -S -emit-llvm {debug_flag} -O0 "
        f"{shlex.quote(to_container_path(source))} "
        f"-o {shlex.quote(to_container_path(llvm_path))}"
    )
    docker_cmd = ["docker", "compose", "run", "--rm", service, "sh", "-c", cmd]
    code, stdout, stderr, elapsed = run_command(docker_cmd)
    return {
        "kind": f"clang-{llvm_version}-docker",
        "toolchain": f"clang-{llvm_version}",
        "safImage": saf_image,
        "command": docker_cmd,
        "exitCode": code,
        "stdout": stdout,
        "stderr": stderr,
        "elapsedMs": elapsed,
    }


def compile_cargo_lib(
    crate_dir: Path,
    out_dir: Path,
    *,
    toolchain: str | None,
) -> tuple[Path | None, dict[str, Any]]:
    """Compile a Rust library crate to LLVM IR and copy the newest `.ll` out."""

    cmd = [
        *rustup_command(toolchain, "cargo"),
        "rustc",
        "--manifest-path",
        str(crate_dir / "Cargo.toml"),
        "--lib",
        "--",
        "--emit=llvm-ir",
        "-C",
        "debuginfo=1",
    ]
    code, stdout, stderr, elapsed = run_command(cmd, cwd=crate_dir)
    compile_info = {
        "kind": "cargo-rustc-lib",
        "toolchain": toolchain or "host-default",
        "command": cmd,
        "exitCode": code,
        "stdout": stdout,
        "stderr": stderr,
        "elapsedMs": elapsed,
    }
    if code != 0:
        return None, compile_info

    candidates = sorted(
        (crate_dir / "target").glob("debug/deps/*.ll"),
        key=lambda p: p.stat().st_mtime,
        reverse=True,
    )
    if not candidates:
        compile_info["exitCode"] = 1
        compile_info["stderr"] += "\nNo target/debug/deps/*.ll produced by cargo rustc."
        return None, compile_info

    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / f"{crate_dir.name}.ll"
    shutil.copy2(candidates[0], target)
    compile_info["copiedFrom"] = str(candidates[0])
    return target, compile_info


def load_previous(path: Path | None) -> dict[str, Any] | None:
    """Load previous facts if present."""

    if not path or not path.exists():
        return None
    return json.loads(path.read_text())


def write_json(path: Path, payload: dict[str, Any]) -> None:
    """Write JSON with stable formatting."""

    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def cache_hit_payload(
    *,
    args: argparse.Namespace,
    out_path: Path,
    llvm_path: Path,
    previous: dict[str, Any],
    compile_info: dict[str, Any],
    llvm_hash: str,
    semantic_llvm_hash: str,
) -> dict[str, Any]:
    """Build a payload for a semantic cache hit and refresh source locations."""

    payload = dict(previous)
    relocated_raw = relocate_cached_saf_findings(
        previous.get("findings", []),
        llvm_path=llvm_path,
        source_path=Path(args.source).resolve() if args.source else None,
    )
    relocated_findings = normalize_saf_findings(relocated_raw)
    functions = parse_llvm_functions(llvm_path)
    attach_findings_to_functions(functions, relocated_findings)
    apply_saf_summaries(
        functions,
        args.source_pattern or list(DEFAULT_SOURCES),
        args.sink_pattern or list(DEFAULT_SINKS),
        relocated_findings,
    )

    payload["generatedAt"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    payload["input"] = {
        **payload.get("input", {}),
        "source": repo_relative(Path(args.source)) if args.source else None,
        "llvmIr": repo_relative(llvm_path),
        "llvmSha256": llvm_hash,
        "semanticLlvmSha256": semantic_llvm_hash,
    }
    payload["functions"] = functions
    payload["findings"] = relocated_findings
    payload.setdefault("timings", {})
    payload["timings"].update(
        {
            "compileMs": compile_info.get("elapsedMs"),
            "safWallMs": 0.0,
            "orchestrationMs": 0.0,
        }
    )
    payload["saf"] = {
        **payload.get("saf", {}),
        "image": args.saf_image,
        "service": SAF_SERVICES[args.saf_image],
        "invoked": False,
        "cacheHit": True,
        "status": "cache-hit",
        "locationRefresh": "current LLVM debug locations",
        "summaryKind": "trace-derived-per-function",
        "summaryFunctionCount": sum(1 for fn in functions if summary_has_content(fn.get("summary"))),
        "directFindingCount": len(relocated_findings),
        "rawFindingCount": len(relocated_raw),
    }
    payload.setdefault("assertions", [])
    payload["assertions"].append(
        {"name": "SAF not re-invoked when LLVM IR hash is unchanged", "pass": True}
    )
    write_json(out_path, payload)
    return payload


def relocate_cached_saf_findings(
    previous_findings: list[dict[str, Any]],
    *,
    llvm_path: Path,
    source_path: Path | None,
) -> list[dict[str, Any]]:
    """Move cached SAF findings onto current source coordinates from LLVM debug info."""

    locations_by_symbol = current_call_locations_by_symbol(llvm_path, source_path)
    relocated: list[dict[str, Any]] = []
    for finding in previous_findings:
        raw = finding.get("safFinding") if isinstance(finding, dict) else None
        if not isinstance(raw, dict):
            continue
        relocated.append(relocate_cached_saf_finding(raw, locations_by_symbol))
    return relocated


def current_call_locations_by_symbol(
    llvm_path: Path,
    source_path: Path | None,
) -> dict[str, list[str]]:
    """Map LLVM direct-call symbols to current source locations."""

    text = llvm_path.read_text(errors="replace")
    debug_locations = llvm_debug_locations(text)
    source_label = str(source_path.resolve()) if source_path else repo_relative(llvm_path)
    locations_by_symbol: dict[str, list[str]] = {}
    pending_call_symbol: str | None = None

    for line in text.splitlines():
        call_match = re.search(
            r'\b(?:call|invoke)\b[^@]*@(?:"([^"]+)"|([^\s(]+))\(',
            line,
        )
        if call_match:
            pending_call_symbol = call_match.group(1) or call_match.group(2)
        debug_match = re.search(r"!\bdbg\s+!(\d+)", line)
        if not debug_match:
            continue
        symbol = (call_match.group(1) or call_match.group(2)) if call_match else pending_call_symbol
        pending_call_symbol = None
        if not symbol:
            continue
        debug_id = int(debug_match.group(1))
        debug_location = debug_locations.get(debug_id)
        if not debug_location:
            continue
        location = f"{source_label}:{debug_location[0]}:{debug_location[1]}"
        key = normalize_call_symbol(symbol)
        locations_by_symbol.setdefault(key, []).append(location)

    return locations_by_symbol


def llvm_debug_locations(text: str) -> dict[int, tuple[int, int]]:
    """Parse LLVM `!DILocation` metadata into line/column pairs."""

    return {
        location_id: (int(record["line"]), int(record["column"]))
        for location_id, record in llvm_debug_location_records(text).items()
    }


def normalize_call_symbol(symbol: Any) -> str:
    """Normalize SAF/LLVM call labels to the raw callee name."""

    text = str(symbol or "").strip()
    text = re.sub(r"^(?:call|invoke)\s+", "", text)
    text = text.strip()
    if text.startswith("@"):
        text = text[1:]
    if text.startswith('"') and text.endswith('"'):
        text = text[1:-1]
    return text


def relocated_location(
    symbol: Any,
    locations_by_symbol: dict[str, list[str]],
    cursors: dict[str, int],
) -> str | None:
    """Return the next current source location for a cached SAF call symbol."""

    key = normalize_call_symbol(symbol)
    if not key:
        return None

    locations = locations_by_symbol.get(key)
    cursor_key = key
    if locations is None:
        for candidate, candidate_locations in locations_by_symbol.items():
            if key in candidate or candidate in key:
                locations = candidate_locations
                cursor_key = candidate
                break
    if not locations:
        return None

    cursor = cursors.get(cursor_key, 0)
    cursors[cursor_key] = cursor + 1
    return locations[min(cursor, len(locations) - 1)]


def relocate_cached_saf_finding(
    finding: dict[str, Any],
    locations_by_symbol: dict[str, list[str]],
) -> dict[str, Any]:
    """Refresh one cached SAF finding's source/sink/trace coordinates."""

    relocated = copy.deepcopy(finding)
    cursors: dict[str, int] = {}
    trace = relocated.get("trace")
    if isinstance(trace, list):
        for idx, step in enumerate(trace):
            if not isinstance(step, dict):
                continue
            if idx == 0:
                from_location = relocated_location(step.get("from_symbol"), locations_by_symbol, cursors)
                if from_location:
                    step["from_location"] = from_location
            to_location = relocated_location(step.get("to_symbol"), locations_by_symbol, cursors)
            if to_location:
                step["to_location"] = to_location

        first_step = next((step for step in trace if isinstance(step, dict)), None)
        last_step = next((step for step in reversed(trace) if isinstance(step, dict)), None)
        if first_step and first_step.get("from_location"):
            relocated["source_location"] = first_step["from_location"]
        if last_step and last_step.get("to_location"):
            relocated["sink_location"] = last_step["to_location"]

    if not relocated.get("source_location"):
        source_location = relocated_location(relocated.get("source_name"), locations_by_symbol, {})
        if source_location:
            relocated["source_location"] = source_location
    if not relocated.get("sink_location"):
        sink_location = relocated_location(relocated.get("sink_name"), locations_by_symbol, {})
        if sink_location:
            relocated["sink_location"] = sink_location
    return relocated


def invoke_inside_docker(
    *,
    llvm_path: Path,
    out_path: Path,
    sources: list[str],
    sinks: list[str],
    sanitizers: list[str],
    saf_image: str,
    ensure_sdk: bool,
) -> tuple[int, str, str, float]:
    """Invoke the SAF SDK path inside Docker."""

    inner_cmd = [
        "python3",
        to_container_path(SCRIPT),
        "--inside",
        "--llvm",
        to_container_path(llvm_path),
        "--out",
        to_container_path(out_path),
        "--saf-image",
        saf_image,
    ]
    for source in sources:
        inner_cmd.extend(["--source-pattern", source])
    for sink in sinks:
        inner_cmd.extend(["--sink-pattern", sink])
    for sanitizer in sanitizers:
        inner_cmd.extend(["--sanitizer-pattern", sanitizer])
    shell_cmd = " ".join(shlex.quote(part) for part in inner_cmd)
    service = SAF_SERVICES[saf_image]
    docker_cmd = ["docker", "compose", "run", "--rm"]
    if saf_image == "llvm22":
        docker_cmd.extend(["-e", "SKIP_MATURIN_BUILD=1"])
    if ensure_sdk:
        build_cmd = (
            "CARGO_TARGET_DIR=/workspace/target-maturin "
            "maturin develop --no-default-features --features llvm-22 "
            "> /tmp/livesaferust-maturin-llvm22.log 2>&1 "
            "|| { cat /tmp/livesaferust-maturin-llvm22.log; exit 1; }"
        )
        shell_cmd = f"{build_cmd} && {shell_cmd}"
    docker_cmd.extend([service, "sh", "-c", shell_cmd])
    return run_command(docker_cmd)


def host_main(args: argparse.Namespace) -> int:
    """Host entry point."""

    if args.saf_image not in SAF_SERVICES:
        raise SystemExit(f"unsupported --saf-image: {args.saf_image}")

    source_patterns = args.source_pattern or list(DEFAULT_SOURCES)
    sink_patterns = args.sink_pattern or list(DEFAULT_SINKS)
    sanitizer_patterns = args.sanitizer_pattern or list(DEFAULT_SANITIZERS)

    out_dir = Path(args.out_dir).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = Path(args.out).resolve() if args.out else out_dir / "saf-facts.json"

    source = Path(args.source).resolve() if args.source else None
    llvm_path: Path | None
    if args.llvm:
        llvm_path = Path(args.llvm).resolve()
        compile_info = {
            "kind": "precompiled-llvm",
            "toolchain": "precompiled",
            "command": [],
            "exitCode": 0,
            "stdout": "",
            "stderr": "",
            "elapsedMs": 0.0,
        }
    elif args.cargo_lib:
        if not source:
            raise SystemExit("--cargo-lib requires a crate directory as source")
        llvm_path, compile_info = compile_cargo_lib(source, out_dir, toolchain=args.rustc)
    else:
        if not source:
            raise SystemExit("source path is required unless --llvm is set")
        llvm_path = out_dir / f"{source.stem}.ll"
        if source.suffix == ".rs":
            if args.rustc_in_docker:
                if not args.rustc:
                    raise SystemExit("--rustc-in-docker requires --rustc <toolchain>")
                compile_info = compile_rust_in_docker(
                    source,
                    llvm_path,
                    toolchain=args.rustc,
                    saf_image=args.saf_image,
                    crate_name=args.crate_name,
                )
            else:
                compile_info = compile_rust(
                    source,
                    llvm_path,
                    toolchain=args.rustc,
                    crate_name=args.crate_name,
                )
        elif source.suffix == ".c":
            compile_info = compile_c_in_docker(
                source,
                llvm_path,
                line_tables=args.c_line_tables,
                saf_image=args.saf_image,
            )
        else:
            raise SystemExit(f"unsupported source extension: {source.suffix}")

    if compile_info["exitCode"] != 0 or not llvm_path or not llvm_path.exists():
        payload = {
            "schema": "livesaferust.saf-backed/0.1",
            "generatedAt": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "input": {"source": repo_relative(source) if source else None},
            "compile": compile_info,
            "saf": {
                "image": args.saf_image,
                "service": SAF_SERVICES[args.saf_image],
                "invoked": False,
                "status": "compile-failed",
                "error": compile_info["stderr"],
            },
            "functions": [],
            "findings": [],
            "assertions": [{"name": "compile to LLVM IR", "pass": False}],
        }
        write_json(out_path, payload)
        return 1

    llvm_hash = sha256_file(llvm_path)
    previous = load_previous(Path(args.previous).resolve() if args.previous else None)
    semantic_hash = semantic_llvm_sha256(llvm_path)
    previous_semantic_hash = previous.get("input", {}).get("semanticLlvmSha256") if previous else None
    if previous and previous_semantic_hash == semantic_hash:
        cache_hit_payload(
            args=args,
        out_path=out_path,
        llvm_path=llvm_path,
        previous=previous,
        compile_info=compile_info,
        llvm_hash=llvm_hash,
        semantic_llvm_hash=semantic_hash,
        )
        print(json.dumps({"status": "cache-hit", "out": str(out_path)}, indent=2))
        return 0

    code, stdout, stderr, saf_elapsed = invoke_inside_docker(
        llvm_path=llvm_path,
        out_path=out_path,
        sources=source_patterns,
        sinks=sink_patterns,
        sanitizers=sanitizer_patterns,
        saf_image=args.saf_image,
        ensure_sdk=args.ensure_saf_sdk,
    )
    payload = load_previous(out_path) or {
        "schema": "livesaferust.saf-backed/0.1",
        "functions": [],
        "findings": [],
    }
    payload["generatedAt"] = time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
    payload["input"] = {
        "source": repo_relative(source) if source else None,
        "llvmIr": repo_relative(llvm_path),
        "llvmSha256": llvm_hash,
        "semanticLlvmSha256": semantic_hash,
    }
    payload["compile"] = compile_info
    payload.setdefault("timings", {})
    payload["timings"]["compileMs"] = compile_info.get("elapsedMs")
    payload["timings"]["safWallMs"] = saf_elapsed
    payload["docker"] = {"exitCode": code, "stdout": stdout, "stderr": stderr}
    payload.setdefault("saf", {})
    payload["saf"].update({
        "image": args.saf_image,
        "service": SAF_SERVICES[args.saf_image],
        "invoked": True,
        "cacheHit": False,
    })
    if code != 0:
        if not payload["saf"].get("error"):
            payload["saf"]["error"] = stderr or stdout
        payload["saf"].update({"status": "failed", "dockerError": stderr or stdout})
    write_json(out_path, payload)

    print(json.dumps({"status": payload.get("saf", {}).get("status"), "out": str(out_path)}, indent=2))
    return 0 if code == 0 else 1


def selector_union(selectors: list[Any]) -> Any:
    """Combine SAF selectors with OR."""

    if not selectors:
        raise ValueError("at least one selector is required")
    result = selectors[0]
    for selector in selectors[1:]:
        result = result | selector
    return result


def inside_main(args: argparse.Namespace) -> int:
    """Container entry point that actually calls the SAF Python SDK."""

    start = now_ms()
    llvm_path = Path(args.llvm)
    out_path = Path(args.out)
    functions = parse_llvm_functions(llvm_path)

    try:
        import saf
        from saf import Project, sanitizers, sinks, sources

        project_start = now_ms()
        project = Project.open(str(llvm_path))
        project_ms = now_ms() - project_start
        query = project.query()

        source_selectors = [sources.call(pattern) for pattern in args.source_pattern]
        sink_selectors = []
        for pattern in args.sink_pattern:
            sink_selectors.append(sinks.call(pattern, arg_index=0))
            # Rust `std::process::Command::new` lowers to an `sret` call where
            # operand 0 is the out pointer and operand 1 is the command program.
            # Adding arg 1 is harmless for C sinks with fewer arguments and makes
            # rustc-emitted std::Command IR visible to SAF.
            sink_selectors.append(sinks.call(pattern, arg_index=1))
        sanitizer_selectors = []
        for pattern in args.sanitizer_pattern:
            sanitizer_selectors.append(sanitizers.call(pattern))
            sanitizer_selectors.append(sanitizers.call(pattern, arg_index=0))
            sanitizer_selectors.append(sanitizers.call(pattern, arg_index=1))

        query_start = now_ms()
        findings = query.taint_flow(
            sources=selector_union(source_selectors),
            sinks=selector_union(sink_selectors),
            sanitizers=selector_union(sanitizer_selectors),
        )
        query_ms = now_ms() - query_start
        finding_dicts = [finding.to_dict() for finding in findings]
        saf_status = "ok"
        saf_error = None

        air = project.air()
        air_names = set(air.function_names())
        for fn in functions:
            fn["inAir"] = fn["name"] in air_names
    except Exception as exc:  # noqa: BLE001 - must capture exact SAF failure.
        project_ms = 0.0
        query_ms = 0.0
        finding_dicts = []
        saf_status = "failed"
        saf_error = f"{type(exc).__name__}: {exc}"

    normalized_findings = normalize_saf_findings(finding_dicts)
    attach_findings_to_functions(functions, normalized_findings)
    apply_saf_summaries(functions, args.source_pattern, args.sink_pattern, normalized_findings)

    payload = {
        "schema": "livesaferust.saf-backed/0.1",
        "frontend": {
            "kind": "saf-python-sdk",
            "parser": "rustc/clang LLVM IR -> SAF LLVM frontend",
        },
        "input": {
            "llvmIr": str(llvm_path),
            "llvmSha256": sha256_file(llvm_path),
            "semanticLlvmSha256": semantic_llvm_sha256(llvm_path),
        },
        "timings": {
            "safProjectOpenMs": project_ms,
            "safQueryMs": query_ms,
            "orchestrationMs": now_ms() - start,
        },
        "saf": {
            "image": args.saf_image,
            "service": SAF_SERVICES.get(args.saf_image, "inside"),
            "invoked": True,
            "cacheHit": False,
            "status": saf_status,
            "error": saf_error,
            "rawFindingCount": len(finding_dicts),
            "directFindingCount": len(normalized_findings),
            "summaryKind": "trace-derived-per-function",
            "summaryFunctionCount": sum(1 for fn in functions if summary_has_content(fn.get("summary"))),
            "sourcePatterns": args.source_pattern,
            "sinkPatterns": args.sink_pattern,
            "sanitizerPatterns": args.sanitizer_pattern,
        },
        "functions": functions,
        "findings": normalized_findings,
        "assertions": [
            {"name": "SAF Python SDK was invoked", "pass": saf_status == "ok"},
        ],
    }
    write_json(out_path, payload)
    return 0 if saf_status == "ok" else 1


def normalize_saf_findings(findings: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Convert SAF findings into LiveSafeRust diagnostic records."""

    normalized = []
    for finding in findings:
        sink_name = finding.get("sink_name") or finding.get("sink_id") or "sink"
        source_name = finding.get("source_name") or finding.get("source_id") or "source"
        sink_location = finding.get("sink_location")
        line = location_line(sink_location) or 1
        source_span = source_span_from_location(sink_location, sink_name)
        rule_id = rule_id_for_sink(str(sink_name))
        normalized.append(
            {
                "ruleId": rule_id,
                "function": containing_function_from_location(sink_location)
                or "saf",
                "line": line,
                "sink": sink_name,
                "message": f"SAF taint flow reaches {sink_name}",
                "trace": trace_labels(finding),
                "sourceSpan": source_span,
                "sourceTrace": source_trace_steps(finding),
                "sourceLine": f"{source_name} -> {sink_name}",
                "safFinding": finding,
            }
        )
    return normalized


def location_line(location: Any) -> int | None:
    """Parse a line number from SAF location strings."""

    if not isinstance(location, str):
        return None
    match = re.search(r":(\d+):\d+$", location)
    return int(match.group(1)) if match else None


def location_column(location: Any) -> int | None:
    """Parse a column number from SAF location strings."""

    if not isinstance(location, str):
        return None
    match = re.search(r":\d+:(\d+)$", location)
    return int(match.group(1)) if match else None


def source_span_from_location(location: Any, label: Any) -> dict[str, Any] | None:
    """Build an LSP-like source span from a SAF SDK location."""

    if not isinstance(location, str):
        return None
    match = re.search(r"^(.*):(\d+):(\d+)$", location)
    if not match:
        return None
    start_line = int(match.group(2))
    start_column = int(match.group(3))
    label_text = str(label or "")
    file_path = match.group(1)
    return {
        "file": file_path,
        "startLine": start_line,
        "startColumn": start_column,
        "endLine": start_line,
        "endColumn": diagnostic_end_column(file_path, start_line, start_column),
        "label": label_text,
        "ruleId": rule_id_for_sink(label_text),
    }


def diagnostic_end_column(file_path: str, line: int, column: int) -> int:
    """Choose a display-only diagnostic underline width from a SAF location."""

    end_column = column + DIAGNOSTIC_UNDERLINE_WIDTH
    try:
        lines = Path(file_path).read_text(errors="replace").splitlines()
        line_text = lines[line - 1]
    except (OSError, IndexError):
        return end_column
    line_end_column = len(line_text) + 1
    return max(column + 1, min(end_column, line_end_column))


def rule_id_for_sink(sink: str) -> str:
    """Classify the direct SAF sink label into a diagnostic rule id."""

    if "File" in sink or "open" in sink or "create" in sink:
        return "tainted-path"
    return "tainted-command"


def containing_function_from_location(location: Any) -> str | None:
    """Best-effort function label from a SAF location string."""

    if not isinstance(location, str):
        return None
    if ":" in location:
        return Path(location.split(":", 1)[0]).stem
    return None


def trace_labels(finding: dict[str, Any]) -> list[str]:
    """Build a compact trace label list."""

    labels: list[str] = []
    for step in finding.get("trace", []):
        for key in ("from_symbol", "to_symbol"):
            value = step.get(key)
            if value and value not in labels:
                labels.append(value)
    if not labels:
        labels = [
            str(finding.get("source_name") or finding.get("source_id") or "source"),
            str(finding.get("sink_name") or finding.get("sink_id") or "sink"),
        ]
    return labels


def source_trace_steps(finding: dict[str, Any]) -> list[dict[str, Any]]:
    """Translate direct SAF trace locations into source-level trace steps."""

    steps: list[dict[str, Any]] = []
    for idx, step in enumerate(finding.get("trace", [])):
        if idx == 0:
            append_trace_endpoint(steps, "source", step.get("from_location"), step.get("from_symbol"))
        append_trace_endpoint(steps, step.get("edge") or "flow", step.get("to_location"), step.get("to_symbol"))
    return steps


def append_trace_endpoint(
    steps: list[dict[str, Any]],
    kind: str,
    location: Any,
    symbol: Any,
) -> None:
    """Append one located SAF trace endpoint if it has a source location."""

    line = location_line(location)
    column = location_column(location)
    if line is None or column is None:
        return
    label = str(symbol or "value")
    steps.append({
        "kind": kind,
        "line": line,
        "column": column,
        "message": f"{label} @ {location}",
        "text": str(location),
    })


def attach_findings_to_functions(functions: list[dict[str, Any]], findings: list[dict[str, Any]]) -> None:
    """Attach SAF findings to the function containing the SAF sink location."""

    for fn in functions:
        fn["findings"] = []
    for finding in findings:
        raw = finding.get("safFinding", {})
        target = function_for_location(functions, raw.get("sink_location")) or function_for_location(
            functions,
            raw.get("source_location"),
        )
        if not target:
            continue
        finding["function"] = target["name"]
        target.setdefault("findings", []).append(finding)


def source_file_from_location(location: Any) -> str | None:
    """Parse the file path from a SAF `file:line:column` location string."""

    if not isinstance(location, str):
        return None
    match = re.search(r"^(.*):\d+:\d+$", location)
    return match.group(1) if match else None


def function_for_location(functions: list[dict[str, Any]], location: Any) -> dict[str, Any] | None:
    """Find the narrowest function whose LLVM debug source range covers a location."""

    line = location_line(location)
    file_path = source_file_from_location(location)
    if line is None or not file_path:
        return None

    candidates: list[tuple[int, dict[str, Any]]] = []
    for fn in functions:
        source_range = fn.get("sourceRange")
        if not source_range:
            continue
        if not paths_match(str(source_range.get("file")), file_path):
            continue
        start = int(source_range.get("startLine") or 0)
        end = int(source_range.get("endLine") or start)
        if start <= line <= end:
            candidates.append((end - start, fn))
    if not candidates:
        return None
    return sorted(candidates, key=lambda item: item[0])[0][1]


def function_for_symbol(functions: list[dict[str, Any]], symbol: Any) -> dict[str, Any] | None:
    """Find the function named by a SAF trace call symbol."""

    key = normalize_call_symbol(symbol)
    if not key:
        return None
    for fn in functions:
        fn_name = normalize_call_symbol(fn.get("name"))
        if key == fn_name or key in fn_name or fn_name in key:
            return fn
    return None


def empty_saf_summary() -> dict[str, list[Any]]:
    """Return the compact summary shape consumed by the LiveSafeRust harness."""

    return {"returnDeps": [], "sinkParams": []}


def summary_has_content(summary: Any) -> bool:
    """Whether a compact SAF summary carries any dependency/sink facts."""

    return bool(summary and (summary.get("returnDeps") or summary.get("sinkParams")))


def add_return_dep(fn: dict[str, Any] | None, dep: str) -> None:
    """Add one return dependency to a function summary."""

    if not fn:
        return
    summary = fn.setdefault("summary", empty_saf_summary())
    summary["returnDeps"] = sorted(set(summary.get("returnDeps", [])) | {dep})


def add_sink_param(fn: dict[str, Any] | None, index: int = 0) -> None:
    """Add one sink parameter to a function summary."""

    if not fn:
        return
    summary = fn.setdefault("summary", empty_saf_summary())
    summary["sinkParams"] = sorted(set(summary.get("sinkParams", [])) | {index})


def pattern_matches(pattern: str, text: str) -> bool:
    """Simple glob-ish match."""

    regex = "^" + re.escape(pattern).replace("\\*", ".*") + "$"
    return re.search(regex, text) is not None or pattern in text


def apply_saf_summaries(
    functions: list[dict[str, Any]],
    source_patterns: list[str],
    sink_patterns: list[str],
    findings: list[dict[str, Any]],
) -> None:
    """Populate per-function summaries from direct SAF findings and traces."""

    for fn in functions:
        fn["summary"] = empty_saf_summary()

    for finding in findings:
        raw = finding.get("safFinding", {})
        if not isinstance(raw, dict):
            continue
        trace = raw.get("trace") if isinstance(raw.get("trace"), list) else []
        first_step = next((step for step in trace if isinstance(step, dict)), None)
        last_step = next((step for step in reversed(trace) if isinstance(step, dict)), None)

        source_symbol = (
            first_step.get("from_symbol")
            if first_step
            else raw.get("source_name") or raw.get("source_id")
        )
        sink_symbol = (
            last_step.get("to_symbol")
            if last_step
            else raw.get("sink_name") or raw.get("sink_id")
        )

        source_fn = function_for_symbol(functions, source_symbol)
        sink_fn = function_for_symbol(functions, sink_symbol)
        source_owner = function_for_location(functions, raw.get("source_location"))
        sink_owner = function_for_location(functions, raw.get("sink_location"))

        if source_fn and any(pattern_matches(pattern, source_fn["name"]) for pattern in source_patterns):
            add_return_dep(source_fn, "SOURCE")
        if sink_fn and any(pattern_matches(pattern, sink_fn["name"]) for pattern in sink_patterns):
            add_sink_param(sink_fn, 0)

        # The owner summary is the IDE-facing summary used by the incremental
        # harness: a local SAF finding means this function has a tainted sink.
        add_sink_param(sink_owner, 0)
        if source_owner and source_owner is not sink_owner:
            add_return_dep(source_owner, "SOURCE")


def build_parser() -> argparse.ArgumentParser:
    """Build CLI parser."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", nargs="?", help="Rust/C source path, or crate dir with --cargo-lib")
    parser.add_argument("--out-dir", default="experiments/livesaferust/out/saf/run")
    parser.add_argument("--out")
    parser.add_argument("--llvm")
    parser.add_argument("--cargo-lib", action="store_true")
    parser.add_argument("--previous")
    parser.add_argument("--inside", action="store_true")
    parser.add_argument("--saf-image", choices=sorted(SAF_SERVICES), default="llvm18")
    parser.add_argument("--ensure-saf-sdk", action="store_true", help="rebuild the SAF Python SDK for the selected image before analysis")
    parser.add_argument("--rustc", help="rustup toolchain to use for Rust LLVM IR generation, e.g. 1.79.0")
    parser.add_argument("--crate-name", help="stable crate name for standalone rustc runs")
    parser.add_argument("--rustc-in-docker", action="store_true", help="compile Rust source inside the selected SAF Docker image")
    parser.add_argument("--c-line-tables", action="store_true", help="compile C fallback with source line tables")
    parser.add_argument("--source-pattern", action="append", default=[])
    parser.add_argument("--sink-pattern", action="append", default=[])
    parser.add_argument("--sanitizer-pattern", action="append", default=[])
    return parser


def main() -> int:
    """Main entry point."""

    args = build_parser().parse_args()
    if not args.source_pattern:
        args.source_pattern = list(DEFAULT_SOURCES)
    if not args.sink_pattern:
        args.sink_pattern = list(DEFAULT_SINKS)
    if not args.sanitizer_pattern:
        args.sanitizer_pattern = list(DEFAULT_SANITIZERS)
    if args.inside:
        return inside_main(args)
    return host_main(args)


if __name__ == "__main__":
    raise SystemExit(main())
