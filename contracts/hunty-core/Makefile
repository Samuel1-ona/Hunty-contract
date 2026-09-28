default: build

all: test

test: build
	cargo test

build:
	stellar contract build
	@ls -l ../../target/wasm32v1-none/release/hunty_core.wasm

fmt:
	cargo fmt --all

clean:
	cargo clean

