# Show recipes
default:
    just -l

# Run this and make sure it works before checking in
ci:
    cargo build
    cargo check
    cargo test
    cargo clippy -- --deny=warnings

# Build WASM files from Rust library
wasm:
    cd kaya_web && wasm-pack build --target=web

# Serve test page locally for testing
dev:
    miniserve kaya_web/index.html



