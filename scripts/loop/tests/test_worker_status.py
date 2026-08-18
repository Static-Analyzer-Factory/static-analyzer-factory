#!/usr/bin/env python3
"""Unit test for lib/worker_status.py extract_result() + the --result subcommand (plan 205 §1a/§3).

extract_result() pulls the worker's cost/turn telemetry from the terminal stream-json `result`
event (plus counts api_retry events and keeps the last assistant text as an advisory summary).
classify_transcript() (the hot rate-limit path) must remain byte-for-byte behaviourally unchanged.
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
LIB = HERE.parent / "lib"
sys.path.insert(0, str(LIB))
import worker_status as ws  # noqa: E402


def _write(path: Path, lines: list) -> str:
    path.write_text("\n".join(json.dumps(x) if isinstance(x, dict) else x for x in lines))
    return str(path)


def test_extract_result_success_pulls_telemetry():
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "system", "subtype": "api_retry", "attempt": 1},
            {"type": "assistant", "message": {"content": [{"type": "text", "text": "working…"}]}},
            {"type": "result", "subtype": "success", "is_error": False, "num_turns": 37,
             "duration_ms": 641203, "duration_api_ms": 588011, "total_cost_usd": 2.14,
             "result": "widened the reach interval"},
        ])
        r = ws.extract_result(p)
        assert r["subtype"] == "success", r
        assert r["num_turns"] == 37, r
        assert abs(r["total_cost_usd"] - 2.14) < 1e-9, r
        assert r["duration_ms"] == 641203, r
        assert r["duration_api_ms"] == 588011, r
        assert r["api_retries"] == 1, r
        assert r["hit_max_turns"] is False, r
        assert r["is_error"] is False, r
        assert r["summary"] == "widened the reach interval", r


def test_extract_result_max_turns_falls_back_to_last_assistant_text():
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "assistant", "message": {"content": [{"type": "text", "text": "first"}]}},
            {"type": "assistant", "message": {"content": [{"type": "text", "text": "still going"}]}},
            {"type": "result", "subtype": "error_max_turns", "is_error": True, "num_turns": 40},
        ])
        r = ws.extract_result(p)
        assert r["hit_max_turns"] is True, r
        assert r["subtype"] == "error_max_turns", r
        assert r["num_turns"] == 40, r
        # no `result` text -> summary is the most recent assistant block
        assert r["summary"] == "still going", r


def test_extract_result_counts_multiple_retries():
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "system", "subtype": "api_retry"},
            {"type": "system", "subtype": "api_retry"},
            {"type": "system", "subtype": "api_retry"},
            {"type": "result", "subtype": "success", "num_turns": 5, "total_cost_usd": 0.1},
        ])
        assert ws.extract_result(p)["api_retries"] == 3


def test_extract_result_missing_transcript_is_zeros():
    r = ws.extract_result("/no/such/transcript.jsonl")
    assert r["num_turns"] == 0 and r["total_cost_usd"] == 0.0 and r["subtype"] is None, r
    assert r["summary"] == "", r


def test_extract_result_tolerates_garbage_lines():
    with tempfile.TemporaryDirectory() as dd:
        p = Path(dd) / "t.jsonl"
        p.write_text('not json\n{"type":"result","subtype":"success","num_turns":3}\n\n{bad\n')
        r = ws.extract_result(str(p))
        assert r["num_turns"] == 3 and r["subtype"] == "success", r


def test_classify_transcript_unchanged_on_success():
    # regression guard: the extract_result addition must not perturb the classifier.
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "result", "subtype": "success", "is_error": False},
        ])
        assert ws.classify_transcript(p) == "success"


def test_cli_result_subcommand_emits_json_and_default_classifies():
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "result", "subtype": "success", "num_turns": 9, "total_cost_usd": 0.5},
        ])
        cli = str(LIB / "worker_status.py")
        out = subprocess.run([sys.executable, cli, "--result", p], capture_output=True, text=True)
        obj = json.loads(out.stdout.strip())
        assert obj["num_turns"] == 9 and obj["subtype"] == "success", obj
        # default (no --result) still prints the classification word
        out2 = subprocess.run([sys.executable, cli, p], capture_output=True, text=True)
        assert out2.stdout.strip() == "success", out2.stdout


def test_classify_transcript_outage_on_503_no_available_accounts():
    # the real leak (arm-14 on cd-vm-15): 4x api_retry server_error/503, then a bogus
    # result:success + is_error=True whose text is the proxy outage message.
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "system", "subtype": "api_retry", "error": "server_error", "error_status": 503},
            {"type": "assistant", "message": {"content": [{"type": "text",
                "text": "API Error: 503 No available accounts: no available accounts. Temporary."}]}},
            {"type": "result", "subtype": "success", "is_error": True, "num_turns": 1,
             "result": "API Error: 503 No available accounts: no available accounts. Temporary."},
        ])
        assert ws.classify_transcript(p) == "outage"


def test_classify_transcript_outage_from_assistant_when_no_result_event():
    # transcript cut off after the assistant outage message, no terminal result
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "assistant", "message": {"content": [{"type": "text",
                "text": "API Error: 503 No available accounts: no available accounts."}]}},
        ])
        assert ws.classify_transcript(p) == "outage"


def test_classify_transcript_genuine_success_quoting_phrase_stays_success():
    # guard: a real success (is_error=False) is NOT clobbered even if it quotes the phrase
    with tempfile.TemporaryDirectory() as dd:
        p = _write(Path(dd) / "t.jsonl", [
            {"type": "result", "subtype": "success", "is_error": False, "num_turns": 22,
             "result": "handled the 503 'no available accounts' path in the proxy client"},
        ])
        assert ws.classify_transcript(p) == "success"


TESTS = [v for k, v in sorted(globals().items()) if k.startswith("test_")]


def main() -> int:
    passed = 0
    for t in TESTS:
        try:
            t()
            print(f"  ok   {t.__name__}")
            passed += 1
        except AssertionError as e:
            print(f"  FAIL {t.__name__}: {e}")
        except Exception as e:  # noqa: BLE001
            print(f"  ERR  {t.__name__}: {type(e).__name__}: {e}")
    print(f"\n{passed}/{len(TESTS)} green")
    return 0 if passed == len(TESTS) else 1


if __name__ == "__main__":
    sys.exit(main())
