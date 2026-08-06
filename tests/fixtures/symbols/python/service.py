"""Python symbol-extraction fixture (FR-051).

A module docstring mentioning def and class must not confuse the adapter.
"""

import pytest


class Config:
    """A container."""

    def is_named(self):
        return bool(self.name)


def parse_config(text):
    return Config()


@pytest.mark.trace("TC-741", "FR-051-AC-1")
def test_parses_config():
    assert parse_config("x") is not None


class TestService:
    def make_fixture(self):
        return parse_config("x")

    @pytest.mark.trace("TC-743")
    def test_rejects_empty(self):
        assert self.make_fixture() is not None
