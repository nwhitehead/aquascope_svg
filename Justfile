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
    cd kaya_web && miniserve .

build:
    cd kaya_web && npm run build

deploy:
    rsync --archive dist/ root@shimmermathlabs.com:/var/www/kaya/

rendertest:
    rm -f test.png && clear && cargo run --bin kaya_png -- testdata/test.states --output test.png && eog test.png