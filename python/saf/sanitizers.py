"""Sanitizer selectors for taint analysis.

Sanitizer selectors identify functions that cleanse tainted data.

Example:
    from saf import sanitizers

    # Select calls to sanitize functions
    san = sanitizers.call("sanitize_input", arg_index=0)

    # Select calls to escape functions
    san = sanitizers.call("html_escape", arg_index=0)

    # Combine multiple sanitizers with |
    combined = sanitizers.call("sanitize") | sanitizers.call("escape")
"""

from saf._saf import call as _call, arg_to as _arg_to, Selector, SelectorSet

__all__ = [
    "call",
    "arg_to",
]


def call(callee: str, *, arg_index: int | None = None) -> Selector:
    """Select calls to a function as a sanitizer.

    When data flows through a sanitizer, the taint is removed.

    Args:
        callee: Callee function name pattern (glob-style or regex).
        arg_index: If specified, only the argument at this index is sanitized.
                   If None, the return value is considered sanitized.

    Returns:
        A Selector for the specified sanitizer.
    """
    if arg_index is not None:
        return _arg_to(callee, arg_index)
    return _call(callee)


def arg_to(callee: str, index: int) -> Selector:
    """Select arguments passed to a sanitizing function.

    Args:
        callee: Callee function name pattern.
        index: Argument index (0-based).

    Returns:
        A Selector for the specified argument.
    """
    return _arg_to(callee, index)
