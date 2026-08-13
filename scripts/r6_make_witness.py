#!/usr/bin/env python3
"""Slice 0e — R6 no-overflow target-only witness generator (measurement only; NO production code).

Faithfully mirrors what SAF's R6 `lower_overflow_hit` + `ViolationWitness::assemble`
WOULD emit: a byte-stable YAML-2.0 violation witness with a single `target` waypoint
at the overflowing operation's source line (identical schema to the R5 memsafety
witness — see witness_yaml.rs golden_target_only_layout). Used ONLY to de-risk whether
CPAchecker/witnesslint confirm a no-overflow target-only witness before writing any
Rust. Usage:

    r6_make_witness.py <program> <line> [col] [data_model=LP64|ILP32] [prp] > witness.yml
"""
import hashlib
import sys
from pathlib import Path


def det_uuid(seed: bytes) -> str:
    d = hashlib.sha256(seed).digest()
    b = bytearray(d[:16])
    b[6] = (b[6] & 0x0F) | 0x50   # version 5
    b[8] = (b[8] & 0x3F) | 0x80   # RFC-4122 variant
    h = b.hex()
    return f"{h[0:8]}-{h[8:12]}-{h[12:16]}-{h[16:20]}-{h[20:32]}"


def main():
    if len(sys.argv) < 3:
        sys.exit("usage: r6_make_witness.py <program> <line> [col] [LP64|ILP32] [prp]")
    prog = Path(sys.argv[1])
    line = int(sys.argv[2])
    col = int(sys.argv[3]) if len(sys.argv) > 3 and sys.argv[3].isdigit() else None
    dm = sys.argv[4] if len(sys.argv) > 4 else "LP64"
    prp = sys.argv[5] if len(sys.argv) > 5 else "tests/benchmarks/sv-benchmarks/c/properties/no-overflow.prp"

    data = prog.read_bytes()
    sha = hashlib.sha256(data).hexdigest()
    base = prog.name
    spec = Path(prp).read_text().strip()
    version = "0.1.0"
    uuid = det_uuid(sha.encode() + spec.encode() + version.encode())

    col_line = f"\n          column: {col}" if col is not None else ""
    # Exact serde layout from witness_yaml.rs (golden_target_only_layout), no `function`.
    print(f"""\
- entry_type: violation_sequence
  metadata:
    format_version: '2.0'
    uuid: {uuid}
    creation_time: 2024-01-01T00:00:00Z
    producer:
      name: SAF
      version: {version}
    task:
      input_files:
      - {base}
      input_file_hashes:
        {base}: {sha}
      specification: {spec}
      data_model: {dm}
      language: C
  content:
  - segment:
    - waypoint:
        action: follow
        type: target
        location:
          file_name: {base}
          line: {line}{col_line}""")


if __name__ == "__main__":
    main()
