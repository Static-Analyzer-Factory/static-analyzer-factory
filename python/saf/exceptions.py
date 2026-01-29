"""Exception types for SAF errors.

All SAF exceptions inherit from SafError and provide structured error information
via .code and .details attributes for programmatic handling.

Example:
    from saf import Project
    from saf.exceptions import FrontendError, SafError

    try:
        proj = Project.open("invalid.air.json")
    except FrontendError as e:
        print(f"Error code: {e.code}")
        print(f"Details: {e.details}")
    except SafError as e:
        print(f"SAF error: {e}")
"""

from saf._saf import (
    SafError,
    FrontendError,
    AnalysisError,
    QueryError,
    ConfigError,
)

__all__ = [
    "SafError",
    "FrontendError",
    "AnalysisError",
    "QueryError",
    "ConfigError",
]
