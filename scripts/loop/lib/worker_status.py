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


def extract_result(path: str) -> dict:
    """Pull worker telemetry from the terminal stream-json `result` event + the last assistant
    message (plan 205 §1a/§3). Pure, deterministic, tolerant of a truncated/missing transcript
    (returns zeros/None). Fields mirror Claude Code's headless `result` event; `summary` is the
    worker's own final text (what it claims it did) — ADVISORY only, NEVER trusted for scoring.
    """
    out = {"subtype": None, "num_turns": 0, "total_cost_usd": 0.0, "duration_ms": 0,
           "duration_api_ms": 0, "is_error": None, "api_retries": 0, "hit_max_turns": False,
           "summary": ""}
    last_assistant = ""
    try:
        lines = open(path, encoding="utf-8", errors="replace").read().splitlines()
    except OSError:
        return out
    for line in lines:
        line = line.strip()
        if not line:
            continue
        try:
            rec = json.loads(line)
        except (ValueError, TypeError):
            continue
        t = rec.get("type")
        if t == "system" and rec.get("subtype") == "api_retry":
            out["api_retries"] += 1
        elif t == "assistant":
            # keep the most recent assistant text block (the worker's running summary)
            for blk in (rec.get("message", {}) or {}).get("content", []) or []:
                if isinstance(blk, dict) and blk.get("type") == "text":
                    last_assistant = blk.get("text", "") or last_assistant
        elif t == "result":
            out["subtype"] = rec.get("subtype")
            out["num_turns"] = rec.get("num_turns", 0) or 0
            out["total_cost_usd"] = rec.get("total_cost_usd", 0.0) or 0.0
            out["duration_ms"] = rec.get("duration_ms", 0) or 0
            out["duration_api_ms"] = rec.get("duration_api_ms", 0) or 0
            out["is_error"] = rec.get("is_error")
            out["hit_max_turns"] = rec.get("subtype") == "error_max_turns"
            # `result` text is present on success; else fall back to the last assistant block
            out["summary"] = (rec.get("result") or last_assistant or "").strip()[:1200]
    if not out["summary"]:
        out["summary"] = last_assistant.strip()[:1200]
    return out


if __name__ == "__main__":
    # backward-compatible default (classification); `--result <transcript>` emits telemetry JSON.
    if len(sys.argv) > 2 and sys.argv[1] == "--result":
        print(json.dumps(extract_result(sys.argv[2])))
    else:
        print(classify_transcript(sys.argv[1]))
