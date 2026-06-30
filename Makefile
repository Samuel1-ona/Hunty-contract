WASM_DIR := target/wasm32v1-none/release
BINDINGS_DIR := bindings
PYTHON ?= python3
DOCS_OUTPUT := docs/contract-api.md

.PHONY: build bindings all clean generate-api-docs check setup-githooks

all: build bindings

build: generate-api-docs
	stellar contract build

generate-api-docs:
	$(PYTHON) scripts/generate_api_docs.py --output $(DOCS_OUTPUT)

bindings: build
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/hunty_core.wasm \
		--output-dir $(BINDINGS_DIR)/hunty-core \
		--overwrite
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/reward_manager.wasm \
		--output-dir $(BINDINGS_DIR)/reward-manager \
		--overwrite
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/nft_reward.wasm \
		--output-dir $(BINDINGS_DIR)/nft-reward \
		--overwrite

check:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings

setup-githooks:
	git config core.hooksPath .githooks

clean:
	cargo clean
