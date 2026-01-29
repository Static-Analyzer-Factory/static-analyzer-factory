"""Type stubs for the native SAF extension module."""

def version() -> str:
    """Return the SAF version string."""
    ...

class Project:
    """A SAF analysis project opened from input files."""

    @staticmethod
    def open(
        path: str,
        cache_dir: str | None = None,
        frontend: str = "llvm",
    ) -> "Project":
        """Open a project from the given path."""
        ...
