"""Deterministic Claude-Code headless rate-limit / result classification for the supervisor.

All decisions are made from the STRUCTURED stream-json events (Addendum A6), never from prose:
the supervisor runs workers with `--output-format stream-json --verbose`, classifies each
`system`/`api_retry` event and the terminal `result` event, and (on a hard usage-window block)
parses the Unix-epoch reset to sleep-to-reset. Behind the VM proxy the CLI's own classification is
untrustworthy, so this module + a conservative fixed-sleep fallback own the policy.
"""
from __future__ import annotations

import re

# Retryable now: the CLI is (or should be) auto-retrying; the supervisor adds jittered backoff.
_TRANSIENT = {"overloaded", "server_error", "rate_limit"}
# Terminal misconfig/quota-per-request: spinning cannot help — park the arm and alert.
_ABORT = {
    "authentication_failed", "billing_error", "oauth_org_not_allowed",
    "invalid_request", "model_not_found", "max_output_tokens",
}

_PIPE_EPOCH = re.compile(r"\|(\d{10})\b")
_RESETS_AT = re.compile(r"resets_at\"?\s*[:=]\s*\"?(\d{10})\b")


def classify_api_retry(event: dict) -> str:
    """Map a stream-json `system`/`api_retry` event to 'transient' | 'abort' | 'unknown'.

    A null `error` with a null `error_status` is a bare connection drop -> transient. Categories we
    don't recognize fall to 'unknown' (conservative bounded retry), which keeps the classifier
    forward-compatible when Anthropic adds a new error string.
    """
    err = event.get("error")
    if err in _TRANSIENT:
        return "transient"
    if err in _ABORT:
        return "abort"
    if not err and event.get("error_status") is None:
        return "transient"
    return "unknown"


def classify_result(event: dict) -> str:
    """Map the terminal `result` event to 'success' | 'max_turns' | 'error' from its `subtype`.

    Gate on this, NOT `is_error` and NOT the exit code: `is_error` stays False on `error_max_turns`,
    silently accepting a truncated run. On 'max_turns' the supervisor resumes from the session_id;
    only 'success' carries the `result` text and is eligible to be kept.
    """
    sub = event.get("subtype")
    if sub == "success":
        return "success"
    if sub == "error_max_turns":
        return "max_turns"
    return "error"


def parse_reset_epoch(line: str) -> int | None:
    """Extract a Unix-epoch reset from a headless hard-block line: a pipe-delimited epoch
    (`...usage limit reached|1762952400`, possibly on a non-JSON line) or a status-line JSON
    `rate_limits.*.resets_at`. Returns None when no epoch is present.
    """
    for rx in (_PIPE_EPOCH, _RESETS_AT):
        m = rx.search(line)
        if m:
            return int(m.group(1))
    return None
