"""Source selectors for taint analysis.

Source selectors identify where tainted data enters the program.

Example:
    from saf import sources

    # Select first parameter of 'process_input' function
    src = sources.function_param("process_input", 0)

    # Select all parameters of any function matching 'read_*'
    src = sources.function_param("read_*")

    # Combine multiple sources with |
    combined = sources.function_param("main", 0) | sources.function_return("getenv")
"""

from saf._saf import function_param, function_return, call, Selector, SelectorSet

__all__ = [
    "function_param",
    "function_return",
    "call",
    "argv",
    "getenv",
]


def argv() -> Selector:
    """Select command-line arguments (argv).

    Equivalent to selecting parameters of the main function.

    Returns:
        A Selector for argv values.
    """
    return function_param("main", None)


def getenv(name: str | None = None) -> Selector:
    """Select environment variable reads.

    Args:
        name: Environment variable name pattern, or None for all.

    Returns:
        A Selector for getenv return values.
    """
    if name is not None:
        # For specific env var, we'd need a more sophisticated selector
        # For now, just match getenv calls
        return call(f"getenv")
    return call("getenv")
