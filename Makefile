WASM_DIR := target/wasm32v1-none/release
BINDINGS_DIR := bindings
PYTHON ?= python3
SHA256 ?= sha256sum
DOCS_OUTPUT := docs/contract-api.md

# The binding-stamping recipe uses bash substring expansion.
SHELL := /bin/bash

.PHONY: build bindings all clean generate-api-docs check check-wasm-abi setup-githooks
.PHONY: build bindings all clean generate-api-docs check setup-githooks benchmark-compare
# macOS compatibility: check for shasum (pre-installed on macOS) and fall back to sha256sum.
ifeq ($(shell uname -s),Darwin)
  SHA256 = shasum -a 256
endif

.PHONY: build bindings all clean generate-api-docs check setup-githooks

all: build bindings

build: generate-api-docs
	stellar contract build --locked

generate-api-docs:
	$(PYTHON) scripts/generate_api_docs.py --output $(DOCS_OUTPUT)

# Compute the SHA-256 hash of a WASM file and stamp it into the binding's
# package.json under `contractHash`, so consumers can verify which build
# produced the bindings.
#
# Only the contract hash is stamped. A `sourceCommit` field would record
# whatever HEAD happens to be at generation time, which differs between a
# local build (the branch head) and the `pull_request` merge commit CI builds,
# so the `git diff --exit-code bindings` check could never pass.
define stamp-binding
	contract_hash=$$($(SHA256) "$(WASM_DIR)/$(1)" | cut -d' ' -f1); \
	pkg="$(BINDINGS_DIR)/$(2)/package.json"; \
	if [ -f "$$pkg" ]; then \
		node -e " \
			var p = require('./$$pkg'); \
			p.contractHash = '$$contract_hash'; \
			delete p.sourceCommit; \
			require('fs').writeFileSync('$$pkg', JSON.stringify(p, null, 2) + '\n'); \
		"; \
		echo "  >> Stamped $$pkg: hash=$${contract_hash::12}…"; \
	fi
endef

bindings: build
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/hunty_core.wasm \
		--output-dir $(BINDINGS_DIR)/hunty-core \
		--overwrite
	$(call stamp-binding,hunty_core.wasm,hunty-core)
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/reward_manager.wasm \
		--output-dir $(BINDINGS_DIR)/reward-manager \
		--overwrite
	$(call stamp-binding,reward_manager.wasm,reward-manager)
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/nft_reward.wasm \
		--output-dir $(BINDINGS_DIR)/nft-reward \
		--overwrite
	$(call stamp-binding,nft_reward.wasm,nft-reward)

check-wasm-abi:
	$(PYTHON) scripts/check_wasm_abi.py

validate-config:
	bash scripts/validate_placeholders.sh

check:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings
	cargo test --workspace --locked
	$(MAKE) validate-config
	$(MAKE) check-wasm-abi

benchmark-compare:
	node scripts/ci/compare_gas_benchmarks.mjs

setup-githooks:
	git config core.hooksPath .githooks

clean:
	cargo clean
