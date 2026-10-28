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
	check-toolchain check-workflows check-src-coupling check-fuzz-policy

# Coverage-guided fuzzing lives in the excluded rust/fuzz/ workspace (own
# lockfile, deny.toml, native allow-list and pinned nightly; plan §1.8,
# the development-only native-code exception approved as plan Q2). Never part
# of the product graph.
FUZZ_DIR := fuzz
FUZZ_MAX_TOTAL_TIME ?= 60

.PHONY: help fmt fmt-check build build-wasm clippy clippy-wasm test test-doc doc \
	deny audit supply-chain policy policy-selftest markdownlint sbom ci \
	check-unsafe check-manifests check-annotations check-annotations-strict check-poam-ids \
	check-advisory-sync check-native-deps check-toolchain check-workflows \
	check-src-coupling check-fuzz-policy fuzz-smoke fuzz-deny \
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

check-annotations-strict: ## As check-annotations, and pending mapping IDs fail
	ANNOTATIONS_STRICT=1 scripts/check-annotations.sh

check-poam-ids: ## Referenced POA&M IDs exist in POAM.md
	scripts/check-poam-ids.sh

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

check-fuzz-policy: ## fuzz/deny.toml differs from deny.toml only in its delta block
	@if [ ! -d $(FUZZ_DIR) ]; then \
		echo "check-fuzz-policy: skip ($(FUZZ_DIR)/ absent; Phase 1 WU9 adds it)"; \
	elif [ ! -x scripts/check-fuzz-policy.sh ]; then \
		echo "check-fuzz-policy: FAIL $(FUZZ_DIR)/ exists but scripts/check-fuzz-policy.sh is missing" >&2; exit 1; \
	else \
		scripts/check-fuzz-policy.sh; \
	fi

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

# The fuzz targets need cargo-fuzz and the nightly pinned in
# fuzz/rust-toolchain.toml (run from rust/; cd fuzz picks up that pin). They
# pass as no-ops while fuzz/ is absent; WU9 makes them real.
fuzz-smoke: ## Each fuzz target for FUZZ_MAX_TOTAL_TIME s (needs cargo-fuzz + pinned nightly)
	@if [ ! -d $(FUZZ_DIR) ]; then \
		echo "fuzz-smoke: skip ($(FUZZ_DIR)/ absent; Phase 1 WU9 adds it)"; \
	else \
		cd $(FUZZ_DIR) && targets=$$(cargo fuzz list) && [ -n "$$targets" ] || { echo "fuzz-smoke: no fuzz targets" >&2; exit 1; }; \
		for t in $$targets; do \
			echo "== fuzz $$t ($(FUZZ_MAX_TOTAL_TIME) s)"; \
			cargo fuzz run "$$t" -- -max_total_time=$(FUZZ_MAX_TOTAL_TIME) -rss_limit_mb=2048 -timeout=1; \
		done; \
	fi

fuzz-deny: ## cargo-deny and cargo-audit for the fuzz workspace
	@if [ ! -d $(FUZZ_DIR) ]; then \
		echo "fuzz-deny: skip ($(FUZZ_DIR)/ absent; Phase 1 WU9 adds it)"; \
	else \
		cargo deny --manifest-path $(FUZZ_DIR)/Cargo.toml --locked check && \
		cargo audit -f $(FUZZ_DIR)/Cargo.lock --deny warnings; \
	fi
