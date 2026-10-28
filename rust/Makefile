# SPDX-License-Identifier: Apache-2.0
#
# TAK-RS build entry point. Run from rust/ so rust-toolchain.toml and
# .cargo/config.toml apply. CI calls these targets; `make ci` is the local
# mirror of the Rust CI gates (markdownlint is separate: it needs npm).
#
# COMPLIANCE MAPPING:
# - NIST 800-53: SA-11 (Developer Testing and Evaluation), SA-15 (Development
#   Process, Standards, and Tools)

SHELL := bash
.SHELLFLAGS := -euo pipefail -c
.DEFAULT_GOAL := help

# Crates built for wasm32-unknown-unknown. check-native-deps.sh uses the same
# list as the roots of its wasm32 graph (and as its default WASM_ROOTS).
# tak-proto and tak-cot build for wasm32 so the web UI can use their types
# (Phase 1 plan §2.A WU2).
WASM_CRATES := tak-api-types tak-web-components tak-proto tak-cot
WASM_PKGS := $(addprefix -p ,$(WASM_CRATES))
WASM_TARGET := wasm32-unknown-unknown

# Exact pin; bump deliberately (verified on the npm registry 2026-10-26).
MARKDOWNLINT_CLI2_VERSION := 0.24.0

POLICY_CHECKS := check-unsafe check-manifests check-annotations check-poam-ids \
	check-toolchain check-workflows check-src-coupling check-fuzz-policy \
	check-poam-markers
# check-poam-markers (NIST-010): every POA&M marker comment cites its ID.
# Required through `make policy` since WU10 assigned IDs to every marker.

# Coverage-guided fuzzing lives in the excluded rust/fuzz/ workspace (own
# lockfile, deny.toml, native allow-list and pinned nightly; plan §1.8,
# the development-only native-code exception approved as plan Q2). Never part
# of the product graph.
# Gate evidence (plan WU9) is 300 s per target; rust-fuzz.yml uses 300.
# Seeds come from parity/corpus/cases (scripts/fuzz-seeds.sh); inputs over
# FUZZ_SEED_MAX_BYTES are skipped (the 8 MiB / 1 MiB / 400 KB limit cases),
# and libFuzzer's max_len is pinned to the same value.
FUZZ_DIR := fuzz
FUZZ_MAX_TOTAL_TIME ?= 60
FUZZ_SEED_MAX_BYTES ?= 65536
FUZZ_TARGET ?=
FUZZ_LIBFUZZER_FLAGS = -max_total_time=$(FUZZ_MAX_TOTAL_TIME) -max_len=$(FUZZ_SEED_MAX_BYTES) \
	-rss_limit_mb=2048 -timeout=1
# Hosts the fuzz graph is checked for (fuzz/native-allowlist.txt); not shipping targets.
FUZZ_NATIVE_TARGETS := x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin

.PHONY: help fmt fmt-check build build-wasm clippy clippy-wasm test test-doc doc \
	deny audit supply-chain policy policy-selftest markdownlint sbom ci \
	check-unsafe check-manifests check-annotations check-annotations-strict check-poam-ids \
	check-poam-markers \
	check-advisory-sync check-native-deps check-toolchain check-workflows \
	check-src-coupling check-fuzz-policy fuzz-build fuzz-seeds fuzz-run fuzz-smoke fuzz-deny \
	parity-build-java parity-test parity-capture-corpus

help: ## List targets
	@grep -E '^[a-zA-Z0-9_-]+:.*## ' $(MAKEFILE_LIST) | \
		awk 'BEGIN { FS = ":.*## " } { printf "  %-22s %s\n", $$1, $$2 }'

fmt: ## Format all crates
	cargo fmt --all

fmt-check: ## Check formatting
	cargo fmt --all --check

build: ## Build the workspace (native)
	cargo build --workspace --all-features --locked

build-wasm: ## Build the wasm32 crates
	cargo build $(WASM_PKGS) --target $(WASM_TARGET) --locked

clippy: ## Clippy, native, all targets, warnings are errors
	cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

clippy-wasm: ## Clippy for the wasm32 crates
	cargo clippy $(WASM_PKGS) --target $(WASM_TARGET) --all-features --locked -- -D warnings

test: ## Unit and integration tests
	cargo test --workspace --all-features --locked --lib --bins --tests

test-doc: ## Doc tests
	cargo test --workspace --all-features --locked --doc

doc: ## Rustdoc, warnings are errors
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked

deny: ## cargo-deny: advisories, bans, licenses, sources
	cargo deny --locked check

audit: ## cargo-audit, warnings are errors
	cargo audit --deny warnings

supply-chain: deny audit check-native-deps check-advisory-sync ## All supply-chain checks

check-unsafe: ## Every unsafe has a SAFETY comment
	scripts/check-unsafe.sh

check-manifests: ## Members inherit lints/licence, publish = false
	scripts/check-manifests.sh

check-annotations: ## Compliance/parity annotations and SPDX headers
	scripts/check-annotations.sh

# Both checks always run, so one failure does not hide the other's output.
check-annotations-strict: ## As check-annotations, and pending mapping IDs fail; plus check-poam-markers
	@rc=0; \
	ANNOTATIONS_STRICT=1 scripts/check-annotations.sh || rc=1; \
	scripts/check-poam-markers.sh || rc=1; \
	exit $$rc

check-poam-ids: ## Referenced POA&M IDs exist in POAM.md
	scripts/check-poam-ids.sh

check-poam-markers: ## Every POA&M marker comment cites a POAM-YYYY-MM-DD-NN ID
	scripts/check-poam-markers.sh

check-advisory-sync: ## deny.toml and audit.toml ignores agree
	scripts/check-advisory-sync.sh

check-native-deps: ## Per-target native-code allow-list
	WASM_ROOTS="$(WASM_CRATES)" scripts/check-native-deps.sh

check-toolchain: ## Toolchain pin consistency
	scripts/check-toolchain.sh

check-workflows: ## Static policy for workflows (parsed YAML; needs ruby)
	scripts/check-workflows.sh

check-src-coupling: ## rust/ reads from src/ or utils/ only where ci-changes.sh knows
	scripts/check-src-coupling.sh

check-fuzz-policy: ## fuzz/ excluded and isolated; fuzz/deny.toml = deny.toml + approved delta; dated nightly
	scripts/check-fuzz-policy.sh

policy: $(POLICY_CHECKS) ## All policy checks (supply-chain ones run in supply-chain)

policy-selftest: ## Self-tests for check-*.sh and ci-*.sh
	scripts/selftest.sh

markdownlint: ## Lint Markdown (needs npm registry access)
	npx --yes markdownlint-cli2@$(MARKDOWNLINT_CLI2_VERSION)

sbom: ## CycloneDX SBOM, best-effort (POAM-2026-10-26-01)
	cargo cyclonedx --format json --all-features

ci: fmt-check clippy clippy-wasm build-wasm test test-doc doc policy policy-selftest supply-chain ## Local mirror of CI

parity-build-java: ## Build the Java reference image (needs Docker/Podman)
	parity/harness.sh build-java image

parity-test: ## Run TAKCL StartupTests against the Java reference
	parity/harness.sh up test StartupTests down

parity-capture-corpus: ## Capture the golden corpus from the Java reference and verify it (needs Docker/Podman)
	parity/harness.sh build-java image capture-corpus verify-corpus

# The fuzz targets need cargo-fuzz (0.13.2) and the nightly pinned in
# fuzz/rust-toolchain.toml; `cd fuzz` makes rustup select that pin (no
# `cargo +toolchain` override). cargo-fuzz has no --locked, so `cargo fetch
# --locked` first fails on a stale fuzz/Cargo.lock. Builds use AddressSanitizer
# (cargo-fuzz default) and debug assertions (overflow checks, as the product
# release profile). Crash inputs land in fuzz/artifacts/<target>/.
fuzz-build: check-fuzz-policy ## Build every fuzz target (pinned nightly, ASan, debug assertions)
	cd $(FUZZ_DIR) && cargo fetch --locked && cargo fuzz build --debug-assertions

# fuzz-run regenerates only its own seed directory (FUZZ_SEED_ARGS, a
# target-specific variable that make passes to the fuzz-seeds prerequisite),
# so `make fuzz-run` for different targets can run concurrently (QA-003).
# fuzz-build is shared and serialised by cargo's build-directory lock.
FUZZ_SEED_ARGS ?=
fuzz-seeds: ## Seed dirs fuzz/seeds/<target> from parity/corpus/cases (capped at FUZZ_SEED_MAX_BYTES)
	FUZZ_SEED_MAX_BYTES=$(FUZZ_SEED_MAX_BYTES) scripts/fuzz-seeds.sh $(FUZZ_DIR)/seeds $(FUZZ_SEED_ARGS)

fuzz-run: FUZZ_SEED_ARGS = --only "$(FUZZ_TARGET)"
fuzz-run: fuzz-build fuzz-seeds ## One target for FUZZ_MAX_TOTAL_TIME s: make fuzz-run FUZZ_TARGET=<name>
	@t="$(FUZZ_TARGET)"; \
	[[ "$$t" =~ ^[a-z0-9_]+$$ ]] || { echo "fuzz-run: set FUZZ_TARGET to one of: $$(cd $(FUZZ_DIR) && cargo fuzz list | tr '\n' ' ')" >&2; exit 2; }; \
	cd $(FUZZ_DIR); \
	list=$$(cargo fuzz list); printf "%s\n" "$$list" | grep -qxF "$$t" || { echo "fuzz-run: unknown target '$$t'" >&2; exit 2; }; \
	mkdir -p "corpus/$$t" "seeds/$$t"; \
	echo "== fuzz $$t ($(FUZZ_MAX_TOTAL_TIME) s)"; \
	cargo fuzz run --debug-assertions "$$t" "corpus/$$t" "seeds/$$t" -- $(FUZZ_LIBFUZZER_FLAGS)

fuzz-smoke: fuzz-build fuzz-seeds ## Every fuzz target for FUZZ_MAX_TOTAL_TIME s each (needs cargo-fuzz + pinned nightly)
	@cd $(FUZZ_DIR); \
	targets=$$(cargo fuzz list); \
	[ -n "$$targets" ] || { echo "fuzz-smoke: no fuzz targets" >&2; exit 1; }; \
	for t in $$targets; do \
		mkdir -p "corpus/$$t" "seeds/$$t"; \
		echo "== fuzz $$t ($(FUZZ_MAX_TOTAL_TIME) s)"; \
		cargo fuzz run --debug-assertions "$$t" "corpus/$$t" "seeds/$$t" -- $(FUZZ_LIBFUZZER_FLAGS); \
	done

# cargo-deny defaults to <cwd>/deny.toml, so the fuzz policy is passed
# explicitly. cargo audit runs from rust/, so .cargo/audit.toml applies (its
# ignores equal deny.toml's, which fuzz/deny.toml shares).
fuzz-deny: check-fuzz-policy ## cargo-deny, cargo-audit and native-code allow-list for the fuzz workspace
	cargo deny --manifest-path $(FUZZ_DIR)/Cargo.toml --locked --config $(FUZZ_DIR)/deny.toml check
	cargo audit -f $(FUZZ_DIR)/Cargo.lock --deny warnings
	RUST_ROOT="$(CURDIR)/$(FUZZ_DIR)" NATIVE_ALLOWLIST="$(CURDIR)/$(FUZZ_DIR)/native-allowlist.txt" \
		NATIVE_TARGETS="$(FUZZ_NATIVE_TARGETS)" WASM_ROOTS= scripts/check-native-deps.sh
