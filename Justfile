# Show recipes
default:
    just -l

# Run this and make sure it works before checking in
ci:
    cargo build
    cargo check
    cargo test
    cargo clippy -- --deny=warnings

wasm:
    cd kaya_web && wasm-pack build --target=web
