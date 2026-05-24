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
	@echo "  make ci               - Per-PR CI gates (fmt-check + lint + test + deny + audit-unsafe + audit-static)"
	@echo ""
	@echo "Hardening (scheduled / pre-tag):"
	@echo "  make cargo-audit      - cargo audit (RUSTSEC advisories)"
	@echo "  make miri             - cargo +nightly miri test --lib (UB detection)"
	@echo "  make mutants          - cargo mutants -p quire-rs --in-place --check"
	@echo "  make fuzz             - 60s smoke run of each cargo-fuzz target"
	@echo "  make audit-static     - Run all scripts/audits/*.sh"
	@echo "  make hardening        - Full pre-tag set: audit-static + cargo-audit + miri + mutants + fuzz"

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

.PHONY: audit-static
audit-static:
	bash scripts/audits/check_no_net_deps.sh
	bash scripts/audits/check_no_schemars.sh
	bash scripts/audits/check_no_shellout.sh
	bash scripts/audits/check_dep_pins.sh
	bash scripts/audits/check_hashmap_audit.sh
	bash scripts/audits/verify_cookiecutter_inheritance.sh

# =============================================================================
# Hardening (scheduled-only in CI; available locally on demand)
# =============================================================================

.PHONY: miri
miri:
	@if ! rustup toolchain list | grep -q nightly; then \
		echo "miri: nightly toolchain not installed. Run: rustup toolchain install nightly --component miri"; \
		exit 1; \
	fi
	$(CARGO) +nightly miri test --lib

.PHONY: mutants
mutants:
	$(CARGO) mutants -p quire-rs --in-place --check

# =============================================================================
# Perf gates (Task 014, NFR-001/002/007)
# =============================================================================

.PHONY: perf-baseline
perf-baseline:
	$(CARGO) bench --bench render --bench parse --bench load -- --save-baseline main

.PHONY: perf-check
perf-check:
	$(CARGO) bench --bench render --bench parse --bench load -- --baseline main
	bash scripts/check_perf_regression.sh

.PHONY: perf-gate
perf-gate:
	bash scripts/check_perf_regression.sh

# =============================================================================
# Parity regen (Task 013)
# =============================================================================

.PHONY: parity-regen
parity-regen:
	bash scripts/regenerate_parity_fixtures.sh

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

.PHONY: ci
ci: fmt-check lint test deny audit-unsafe audit-static

.PHONY: hardening
hardening: audit-static cargo-audit miri mutants fuzz
