"""Run LeakSanitizer while CPython's runtime objects are still reachable."""

import ctypes


# implements: NFR-018-AC-2
def pytest_sessionfinish(session, exitstatus):
    """Check test-created leaks before CPython teardown obscures their origin.

    CPython's process-shutdown path releases interpreter-owned Unicode objects
    after test code has finished. Running LeakSanitizer here avoids treating
    those runtime allocations as test leaks, while allocations leaked by the
    extension during the tests are still reported. LeakSanitizer's explicit
    check is one-shot; it replaces the less-informative automatic exit check.
    """
    try:
        leak_check = ctypes.CDLL(None).__lsan_do_leak_check
    except AttributeError as exc:
        raise RuntimeError("AddressSanitizer runtime has no LeakSanitizer entry point") from exc
    leak_check.argtypes = []
    leak_check.restype = None
    leak_check()
