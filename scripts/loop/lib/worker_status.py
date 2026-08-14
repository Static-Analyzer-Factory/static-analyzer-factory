#!/usr/bin/env python3
"""Classify a captured worker stream-json transcript into one supervisor action word:

    success | max_turns | sleep:<epoch> | transient | abort | error

Deterministic, from STRUCTURED events only (Addendum A6). Precedence: a clean terminal
`result: success` wins; otherwise a hard usage-window block (a reset epoch) means sleep-to-reset;
otherwise an abort category; otherwise max_turns (resume); otherwise a transient throttle (backoff).
"""
from __future__ import annotations

import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ratelimit as rl  # noqa: E402


def classify_transcript(path: str) -> str:
    result_kind = None
    hard_epoch = None
    saw_abort = False
    saw_transient = False
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    except OSError:
        return "error"
    for line in lines:
        line = line.strip()
        if not line:
            continue
        ep = rl.parse_reset_epoch(line)
        if ep and ("limit" in line.lower() or "resets" in line.lower()):
            hard_epoch = ep
        try:
            rec = json.loads(line)
        except (ValueError, TypeError):
            continue
        t = rec.get("type")
        if t == "system" and rec.get("subtype") == "api_retry":
            c = rl.classify_api_retry(rec)
            if c == "abort":
                saw_abort = True
            elif c == "transient":
                saw_transient = True
        elif t == "result":
            result_kind = rl.classify_result(rec)

    if result_kind == "success":
        return "success"
    if hard_epoch:
        return f"sleep:{hard_epoch}"
    if saw_abort:
        return "abort"
    if result_kind == "max_turns":
        return "max_turns"
    if saw_transient:
        return "transient"
    return "error"


if __name__ == "__main__":
    print(classify_transcript(sys.argv[1]))
