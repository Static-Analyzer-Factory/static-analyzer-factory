"""Sink selectors for taint analysis.

Sink selectors identify where tainted data should not flow (security-sensitive operations).

Example:
    from saf import sinks

    # Select calls to printf with tainted format string
    sink = sinks.call("printf", arg_index=0)

    # Select any call to exec functions
    sink = sinks.call("exec*")

    # Combine multiple sinks with |
    combined = sinks.call("printf", arg_index=0) | sinks.call("system", arg_index=0)
"""

from saf._saf import call as _call, arg_to as _arg_to, Selector, SelectorSet

__all__ = [
    "call",
    "arg_to",
]


def call(callee: str, *, arg_index: int | None = None) -> Selector:
    """Select calls to a function as a sink.

    Args:
        callee: Callee function name pattern (glob-style or regex).
        arg_index: If specified, only match the argument at this index.
                   If None, matches the call result.

    Returns:
        A Selector for the specified call sink.
    """
    if arg_index is not None:
        return _arg_to(callee, arg_index)
    return _call(callee)


def arg_to(callee: str, index: int) -> Selector:
    """Select arguments passed to a specific function.

    Args:
        callee: Callee function name pattern.
        index: Argument index (0-based).

    Returns:
        A Selector for the specified argument.
    """
    return _arg_to(callee, index)
