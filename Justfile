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
    cd kaya_ts && wasm-pack build --target=web && node ts/build.ts

# Serve web app locally for testing
dev:
    cd kaya_web && npm run dev

# Build vue demo app for deployment
build_website:
    cd kaya_web && npm run build

# Deploy to website (assumes webserver already configured)
deploy:
    rsync --archive kaya_web/dist/ root@shimmermathlabs.com:/var/www/kaya/

# Generate some PNG files to test rendering
rendertest:
    rm -f kaya_lib/test_*.png && clear && cargo test && eog kaya_lib/

# Regenerate gold testing files
regen_gold:
    cd kaya_tools/scripts && ./generate_gold.sh
