serve:
    trunk serve

build:
    trunk build --release

test:
    wasm-pack test --node

lint: check fmt clippy

check:
    cargo +nightly check

fmt:
    cargo +nightly fmt

clippy:
    cargo +nightly clippy --workspace --all-targets --tests -- -D warnings
