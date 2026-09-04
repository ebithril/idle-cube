#!/bin/bash

# Build script for WebAssembly deployment

echo "Building Speedcube Idle for WebAssembly..."

# Build for wasm32 target
cargo build --release --target wasm32-unknown-unknown

# Check if build succeeded
if [ $? -eq 0 ]; then
    echo "Build successful!"

    # Add a small delay to prevent race condition
    sleep 1

    # Copy WASM file to root directory
    cp target/wasm32-unknown-unknown/release/idle-cube.wasm idle_cube.wasm

    echo "WASM file copied to current directory"
    echo ""
    echo "To run the game locally:"
    echo "  1. Start a local server:"
    echo "     python3 -m http.server 8080"
    echo "     OR"
    echo "     basic-http-server ."
    echo ""
    echo "  2. Open your browser to:"
    echo "     http://localhost:8080"
else
    echo "Build failed!"
    exit 1
fi
