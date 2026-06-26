WASM_DIR := target/wasm32v1-none/release
BINDINGS_DIR := bindings

.PHONY: build bindings all clean

all: build bindings

build:
	stellar contract build

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

clean:
	cargo clean
