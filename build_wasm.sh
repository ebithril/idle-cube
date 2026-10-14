#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

cargo build --locked --release --target wasm32-unknown-unknown
mkdir -p dist
cp target/wasm32-unknown-unknown/release/idle-cube.wasm dist/idle_cube.wasm
cp index.html web/mq_js_bundle.js dist/
cp web/LICENSE-MIT web/LICENSE-APACHE dist/
touch dist/.nojekyll

printf 'Built dist/. Preview with: python3 -m http.server 8080 --directory dist\n'
