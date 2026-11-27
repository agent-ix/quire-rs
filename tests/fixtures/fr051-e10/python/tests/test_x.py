import os

import pytest


@pytest.mark.trace("FR-051-AC-47")
def test_top():
    assert 2 + 2 == 4


class TestC:
    @pytest.mark.trace("FR-051-AC-47")
    def test_m(self):
        assert True


class TestOuter:
    class TestInner:
        @pytest.mark.trace("FR-051-AC-47")
        def test_n(self):
            assert "trace" == "trace"


@pytest.mark.parametrize("value", [1])
@pytest.mark.trace("FR-051-AC-47", "FR-051-AC-48")
def test_param(value):
    if os.environ.get("IX_TRACE_E2E_FAIL"):
        pytest.fail("intentional E-10 failure")
    assert value == 1
