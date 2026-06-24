WASM_DIR := target/wasm32-unknown-unknown/release
BINDINGS_DIR := bindings

.PHONY: build bindings all clean

all: build bindings

build:
	stellar contract build

bindings: build
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/hunty_core.wasm \
		--output-dir $(BINDINGS_DIR)/hunty-core \
		--contract-name hunty-core
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/reward_manager.wasm \
		--output-dir $(BINDINGS_DIR)/reward-manager \
		--contract-name reward-manager
	stellar contract bindings typescript \
		--wasm $(WASM_DIR)/nft_reward.wasm \
		--output-dir $(BINDINGS_DIR)/nft-reward \
		--contract-name nft-reward

clean:
	cargo clean
