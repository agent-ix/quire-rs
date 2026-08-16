# =============================================================================
# quire-rs Makefile
# =============================================================================

CARGO ?= cargo

# Fuzz targets (must mirror .github/workflows/fuzz.yml matrix).
FUZZ_TARGETS := \
	fuzz_parse_document \
	fuzz_extract_frontmatter \
	fuzz_apply_patch \
	fuzz_extract_dsl \
	fuzz_load_manifest \
	fuzz_load_schema

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make audit-property   - Enforce FR-052-CON-1: no GrammarFinding in the property classifier"
	@echo "  make ci               - Per-PR CI gates (fmt-check + lint + test + deny + audit-unsafe + audit-property + audit-static)"
	@echo ""
	@echo "Hardening (scheduled / pre-tag):"
	@echo "  make cargo-audit      - cargo audit (RUSTSEC advisories)"
	@echo "  make mutants          - cargo mutants -p quire-rs --in-place --check"
	@echo "  make fuzz             - 60s smoke run of each cargo-fuzz target"
	@echo "  make audit-static     - Run all scripts/audits/*.sh"
	@echo "  make hardening        - Full pre-tag set: audit-static + cargo-audit + mutants + fuzz + loom"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets -- -D warnings

.PHONY: test
test:
	$(CARGO) test

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

.PHONY: audit-property
audit-property:
	bash scripts/check_property_purity.sh

.PHONY: audit-static
audit-static:
	bash scripts/audits/check_no_net_deps.sh
	bash scripts/audits/check_no_schemars.sh
	bash scripts/audits/check_no_shellout.sh
	bash scripts/audits/check_dep_pins.sh
	bash scripts/audits/check_hashmap_audit.sh
	bash scripts/audits/check_no_shared_mutable.sh
	bash scripts/audits/verify_cookiecutter_inheritance.sh

# =============================================================================
# Hardening (scheduled-only in CI; available locally on demand)
# =============================================================================

# miri target retired — ADR 0006 (first-party UB is compile-impossible via
# forbid(unsafe_code); dependency advisories via cargo-audit; concurrency via loom).

.PHONY: mutants
mutants:
	$(CARGO) mutants -p quire-rs --in-place --check

# =============================================================================
# Perf gates (Task 014, NFR-001/002/007)
# =============================================================================

.PHONY: perf-baseline
perf-baseline:
	$(CARGO) bench --bench parse --bench load -- --save-baseline main

.PHONY: perf-check
perf-check:
	$(CARGO) bench --bench parse --bench load -- --baseline main
	bash scripts/check_perf_regression.sh

.PHONY: perf-gate
perf-gate:
	bash scripts/check_perf_regression.sh

.PHONY: fuzz
fuzz:
	@if ! rustup toolchain list | grep -q nightly; then \
		echo "fuzz: nightly toolchain not installed. Run: rustup toolchain install nightly && cargo install cargo-fuzz"; \
		exit 1; \
	fi
	@for t in $(FUZZ_TARGETS); do \
		echo "==> fuzzing $$t for 60s"; \
		$(CARGO) +nightly fuzz run $$t -- -max_total_time=60 || exit $$?; \
	done

# =============================================================================
# Composite
# =============================================================================

.PHONY: loom
loom:
	RUSTFLAGS="--cfg loom" cargo test --test concurrency

.PHONY: sanitize
sanitize:
	@echo "TSAN + ASAN on the rayon walk (NFR-018). Needs nightly + build-std."
	@# Pin RUSTC to nightly: a stable rustc on PATH (e.g. homebrew) would
	@# reject the -Zsanitizer probe otherwise.
	NRUSTC=$$(rustup which --toolchain nightly rustc); TGT=$$(rustc -vV | sed -n 's/host: //p'); \
	RUSTC=$$NRUSTC RUSTFLAGS="-Zsanitizer=thread" rustup run nightly cargo test \
		-Z build-std --target $$TGT --test corpus_concurrency
	NRUSTC=$$(rustup which --toolchain nightly rustc); TGT=$$(rustc -vV | sed -n 's/host: //p'); \
	RUSTC=$$NRUSTC RUSTFLAGS="-Zsanitizer=address" rustup run nightly cargo test \
		-Z build-std --target $$TGT --test corpus_concurrency
	@echo "NOTE: TSAN/ASAN of the GIL window + Python object handoff needs a"
	@echo "sanitizer-instrumented CPython and runs on the scheduled CI lane."

.PHONY: coverage-baseline-update
coverage-baseline-update:
	@echo "Regenerating the FR-050-AC-7 coverage baseline (CR-057)."
	@echo "The resulting diff belongs in the pull request — a change to what"
	@echo "coverage reconciles is reviewable, not absorbable."
	QUIRE_UPDATE_COVERAGE_BASELINE=1 cargo test --test coverage_baseline \
		tc824_coverage_report_matches_the_checked_in_baseline
	@git --no-pager diff --stat -- tests/fixtures/coverage_baseline/expected.json

.PHONY: ci
ci: fmt-check lint test deny audit-unsafe audit-property audit-static

# =============================================================================
# Python wheel / sdist + local-publish (pypi.ix)
# =============================================================================

LOCAL_PYPI_URL ?= http://pypi.ix/root/dev/

.PHONY: wheel
wheel:
	maturin build --release --features python --out dist

.PHONY: sdist
sdist:
	maturin sdist --out dist

.PHONY: pytest
pytest:
	pytest tests/python/ -v

# The PyO3-parity gate. `ci` cannot run the binding suite — it needs a built
# wheel — so this is the target that proves FR-042-AC-10 / FR-043-AC-7 /
# FR-047-AC-9 (TC-666, TC-673, TC-715). Required before merging any change to
# src/grammar/, src/python/, or tests/python/: without it those tests rot
# silently against a renamed check id, which is exactly what happened to
# TC-715 across CR-014.
.PHONY: ci-python
ci-python: wheel
	pip install --force-reinstall --no-deps --no-index --find-links dist quire
	pytest tests/python/ -q

# Publish the abi3 wheel + sdist to the local devpi index (pypi.ix).
# Mirrors the filament-* `make local-publish` convention.
.PHONY: local-publish
local-publish: wheel sdist
	@echo "📦 Publishing quire to local PyPI ($(LOCAL_PYPI_URL))..."
	@devpi use $(LOCAL_PYPI_URL)
	@devpi login root --password=''
	@devpi upload --from-dir dist/
	@echo "Published quire to $(LOCAL_PYPI_URL)"

.PHONY: hardening
hardening: audit-static cargo-audit mutants fuzz loom sanitize
