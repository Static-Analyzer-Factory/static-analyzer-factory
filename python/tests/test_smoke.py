"""Smoke tests for the SAF Python package."""


def test_import():
    """Verify the saf package can be imported."""
    import saf

    assert hasattr(saf, "version")
    assert hasattr(saf, "Project")


def test_version():
    """Verify version() returns a non-empty string."""
    from saf import version

    v = version()
    assert isinstance(v, str)
    assert len(v) > 0
    assert v == "0.1.0"
