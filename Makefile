WASM_DIR := target/wasm32-unknown-unknown/release
BINDINGS_DIR := bindings
PYTHON ?= python3
SHA256 ?= sha256sum
DOCS_OUTPUT := docs/contract-api.md

.PHONY: build bindings all clean generate-api-docs check check-wasm-abi setup-githooks
.PHONY: build bindings all clean generate-api-docs check setup-githooks benchmark-compare
# macOS compatibility: check for shasum (pre-installed on macOS) and fall back to sha256sum.
ifeq ($(shell uname -s),Darwin)
  SHA256 = shasum -a 256
endif

.PHONY: build bindings all clean generate-api-docs check setup-githooks

all: build bindings

build: generate-api-docs
	cargo build --workspace --target wasm32-unknown-unknown --release

generate-api-docs:
	$(PYTHON) scripts/generate_api_docs.py --output $(DOCS_OUTPUT)

# Compute SHA-256 hash of a WASM file and retrieve the current git commit.
# Stamps both into the binding's package.json under `contractHash` and
# `sourceCommit` so consumers can verify which build produced the bindings.
define stamp-binding
	contract_hash=$$($(SHA256) "$(WASM_DIR)/$(1)" | cut -d' ' -f1); \
	commit_hash=$$(git rev-parse HEAD 2>/dev/null || echo "unknown"); \
	pkg="$(BINDINGS_DIR)/$(2)/package.json"; \
	if [ -f "$$pkg" ]; then \
		node -e " \
			var p = require('./$$pkg'); \
			p.contractHash = '$$contract_hash'; \
			p.sourceCommit = '$$commit_hash'; \
			require('fs').writeFileSync('$$pkg', JSON.stringify(p, null, 2) + '\n'); \
		"; \
		echo "  >> Stamped $$pkg: hash=$${contract_hash::12}… commit=$${commit_hash::12}…"; \
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
