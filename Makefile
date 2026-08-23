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
	@echo "  make validate         - Validate spec/ with the working-tree engine (#212 gate)"
	@echo "  make ci               - Per-PR CI gates (fmt-check + lint + test + deny + audit-unsafe + audit-property + audit-static + validate)"
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

# The `python` feature is gated off in every target above, so nothing in `ci`
# ever compiles `src/python/`. FR-056 added a field to `GrammarVocabularies`,
# two PyO3 struct literals were not updated, and quire-rs shipped v0.29.0 AND
# v0.30.0 unable to build a wheel at all — discovered only when the wheel job
# was finally dispatched. CLAUDE.md already says a `src/grammar/` change must
# pass `make ci-python`; nothing enforced it.
#
# This is the cheap half: it type-checks the binding without needing a built
# wheel or an interpreter, so a missing field can no longer reach a tag. It
# does NOT replace `make ci-python`, which runs the binding suite and is still
# the only verification of the PyO3-parity criteria.
#
# Its own CARGO_TARGET_DIR on purpose: `--features python` resolves a different
# feature set, and sharing the default target dir makes the next `cargo test`
# link against artifacts built for the other set — which surfaces as bogus
# "trait Serialize is not implemented" errors on types that plainly derive it.
.PHONY: check-python
check-python:
	CARGO_TARGET_DIR=target/python-check $(CARGO) check --features python --quiet

# The scripts/ tooling test suite (#217, #219). `check-python` is a cargo
# type-check of the PyO3 binding and collects no Python tests, so the sweep
# harness and corpus rules are verified here.
.PHONY: check-scripts
check-scripts:
	python3 -m pytest scripts/tests -q

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

# agent-ix/quoin#48 pilot: mutation-score one requirement instead of the whole
# crate. The file set is *computed*, not guessed — `mutants_scope` resolves
# `FR → {AC} → {TC} → {file}` through the module's declared reference columns
# (FR-049/FR-050) and the FR-051 `verifies` relations.
#
# Two hops, not one: `verifies` binds symbols to **TC** ids, so the
# requirement→test edge lives in the Test Matrix and cannot be read off the
# symbol graph alone.
#
# Usage: make mutants-fr FR=FR-026
.PHONY: mutants-fr
mutants-fr:
	@test -n "$(FR)" || { echo "usage: make mutants-fr FR=FR-026" >&2; exit 2; }
	@files=$$($(CARGO) run --release --quiet --example mutants_scope -- $(FR) --files-only); \
	if [ -z "$$files" ]; then \
	  echo "$(FR): no mutable file in the traced set — nothing to mutate." >&2; \
	  $(CARGO) run --release --quiet --example mutants_scope -- $(FR) >&2; \
	  exit 1; \
	fi; \
	echo "$(FR) mutation scope:"; echo "$$files" | sed 's/^/  /'; \
	args=""; for f in $$files; do args="$$args --file $$f"; done; \
	$(CARGO) mutants -p quire-rs $$args

.PHONY: mutants-scope
mutants-scope:
	@test -n "$(FR)" || { echo "usage: make mutants-scope FR=FR-026" >&2; exit 2; }
	@$(CARGO) run --release --quiet --example mutants_scope -- $(FR)

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

# The corpus-scale benchmark (quire-rs#231, CR-099) — `coverage-baseline`'s
# byte-diff pattern extended from one fixture to the whole corpus.
#
# Ratchet, not threshold: a run may only match-or-beat the checked-in value,
# and lowering one requires `bench-update` plus a written justification in the
# PR. A hand-picked threshold invites the number to be tuned to it, which is
# how `ac:unclassifiable` came to pass 99.2% of corpus cells (CR-019).
#
# Needs `quire` on PATH and a module; a corpus entry that cannot be read is
# SKIPPED loudly, never scored 0 — a missing corpus scored as zero is the
# silent-zero defect this benchmark exists to catch.
BENCH_MODULE ?= $(HOME)/dev/spec-artifacts-process/spec_artifacts_process
.PHONY: bench
bench:
	python3 scripts/bench.py --module $(BENCH_MODULE)

# Deliberate regeneration. The diff belongs in the pull request.
.PHONY: bench-update
bench-update:
	python3 scripts/bench.py --update --module $(BENCH_MODULE)

.PHONY: coverage-baseline-update
coverage-baseline-update:
	@echo "Regenerating the FR-050-AC-7 coverage baseline (CR-057)."
	@echo "The resulting diff belongs in the pull request — a change to what"
	@echo "coverage reconciles is reviewable, not absorbable."
	QUIRE_UPDATE_COVERAGE_BASELINE=1 cargo test --test coverage_baseline \
		tc824_coverage_report_matches_the_checked_in_baseline
	@git --no-pager diff --stat -- tests/fixtures/coverage_baseline/expected.json

# The #212 gate: the engine under test validates its own spec/ tree. PR #204
# corrupted a spec/tests.md row, every target below stayed green, and the
# corruption shipped inside v0.41.0 — nothing here ever ran structural
# validation against this repo's own matrix. Runs the working-tree engine
# (cargo run --example), never an installed `quire` CLI, which lags the branch
# under test. Module resolution follows the CLI (IX_FILAMENT_MODULES_PATH /
# ~/.ix/filament/modules). Local gate only: CI workflows stay tag/dispatch.
.PHONY: validate
validate:
	$(CARGO) run --quiet --example spec_validate

# The #265 gate: the engine a consumer LINKS is the engine in this tree.
#
# `quire-cli/Cargo.toml` pins this crate independently of this crate's identity
# and nothing compared the two. Measured: the installed CLI 0.29.0 pinned engine
# v0.42.0 while `binding_census` landed in v0.43.0, so four battletest passes
# reported figures from a binary that could not emit the one signal saying
# whether the binder read a test. Direct analogue of
# `quoin/scripts/check-version-agreement.mjs`, applied to the seam that one
# never covered.
#
# Skipped, loudly, when the consumer is not checked out beside this repo: a
# missing sibling is an environment fact, not drift, and failing on it would
# make `ci` unrunnable for anyone who cloned one repository.
# `--build --require` is the WHOLE GATE, not an option. Without them the check
# compares a manifest to a tree and nothing more — and a pin to v0.42.0, which
# is exactly the incident above, is an ancestor of HEAD and passes. The
# capability tokens are what make distance a verdict: a pinned engine that
# cannot emit `binding_census` fails here, by name, whatever its version says.
QUIRE_CLI ?= ../quire-cli
ENGINE_CAPABILITIES ?= binding_census metrics_envelope

.PHONY: check-engine
check-engine:
	@if [ -f "$(QUIRE_CLI)/Cargo.toml" ]; then \
		python3 scripts/check_engine.py --consumer "$(QUIRE_CLI)" --build \
			$(foreach token,$(ENGINE_CAPABILITIES),--require $(token)); \
	else \
		echo "check_engine: SKIP — no consumer workspace at $(QUIRE_CLI)"; \
	fi

.PHONY: ci
ci: fmt-check lint check-python check-scripts test deny audit-unsafe audit-property audit-static validate check-engine

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
