# Show recipes
default:
    just -l

# Run this and make sure it works before checking in
ci:
    cargo build -j1
    cargo check -j1
    cargo test -j1
    cargo clippy -j1 -- --deny=warnings

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
    rm -f kaya_png/test_*.png && clear && cargo test && eog kaya_png/