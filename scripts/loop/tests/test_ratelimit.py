#!/usr/bin/env python3
"""Plain-python3 unit tests for the loop's rate-limit classifier (Addendum A6, verified specifics)."""
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "lib"))
import ratelimit as rl  # noqa: E402


def test_classify_api_retry_transient_vs_abort():
    # transient -> jittered backoff + resume; the CLI is auto-retrying
    assert rl.classify_api_retry({"error": "overloaded", "error_status": 529}) == "transient"
    assert rl.classify_api_retry({"error": "rate_limit", "error_status": 429}) == "transient"
    assert rl.classify_api_retry({"error": "server_error", "error_status": 500}) == "transient"
    assert rl.classify_api_retry({"error": None, "error_status": None}) == "transient"  # conn drop
    # hard/abort -> park the arm, do not spin
    assert rl.classify_api_retry({"error": "authentication_failed", "error_status": 401}) == "abort"
    assert rl.classify_api_retry({"error": "billing_error", "error_status": 402}) == "abort"
    assert rl.classify_api_retry({"error": "oauth_org_not_allowed", "error_status": 403}) == "abort"
    assert rl.classify_api_retry({"error": "invalid_request", "error_status": 400}) == "abort"
    assert rl.classify_api_retry({"error": "model_not_found", "error_status": 404}) == "abort"
    assert rl.classify_api_retry({"error": "max_output_tokens", "error_status": None}) == "abort"
    # unknown -> conservative bounded retry (version-stable: ignore categories we don't recognize)
    assert rl.classify_api_retry({"error": "brand_new_category", "error_status": 599}) == "unknown"


def test_classify_result_subtype_is_authoritative_not_is_error():
    assert rl.classify_result({"type": "result", "subtype": "success", "is_error": False}) == "success"
    # THE FOOTGUN: is_error stays False on error_max_turns -> must be caught as truncation, not success
    assert rl.classify_result({"type": "result", "subtype": "error_max_turns", "is_error": False}) == "max_turns"
    assert rl.classify_result({"type": "result", "subtype": "error_during_execution", "is_error": True}) == "error"
    assert rl.classify_result({"type": "result", "subtype": "error_max_budget_usd"}) == "error"
    assert rl.classify_result({"type": "result", "subtype": "error_max_structured_output_retries"}) == "error"


def test_parse_reset_epoch_from_pipe_or_statusline():
    # headless hard-block: a pipe-delimited Unix epoch (may be on a non-JSON line)
    assert rl.parse_reset_epoch("usage limit reached|1762952400") == 1762952400
    # status-line JSON exposes rate_limits.*.resets_at as clean epoch seconds
    assert rl.parse_reset_epoch('{"rate_limits":{"five_hour":{"resets_at":1762952400}}}') == 1762952400
    # ordinary lines / short numbers are not epochs
    assert rl.parse_reset_epoch("just a normal log line") is None
    assert rl.parse_reset_epoch("retry_delay_ms=3000") is None


def test_is_account_outage_needs_503_and_phrase():
    # the real proxy-capacity leak -> outage (nudge-resume, don't give up)
    assert rl.is_account_outage("API Error: 503 No available accounts: no available accounts.") is True
    assert rl.is_account_outage("503 no available accounts") is True
    # NOT this outage: a normal summary, a bare 503, or the phrase without 503
    assert rl.is_account_outage("widened the reach interval to [0, 2^30)") is False
    assert rl.is_account_outage("got a 503 server_error, retrying") is False   # no phrase
    assert rl.is_account_outage("no available accounts") is False              # no 503
    assert rl.is_account_outage("") is False
    assert rl.is_account_outage(None) is False


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
