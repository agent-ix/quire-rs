def pytest_configure(config):
    config.addinivalue_line(
        "markers", "trace(*ids): spec trace ids the test verifies (FR-051)"
    )
