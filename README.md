# Speedcube Idle

An idle/incremental game based on speedcubing progression, featuring a shader-rendered 3D cube built with Rust and macroquad.

## Features

- **Shader-Rendered Cube**: 3D cube with custom vertex and fragment shaders featuring dynamic colorful effects
- **Idle Game Mechanics**:
  - Click the cube to solve it and earn points
  - Unlock auto-solving to generate passive income
  - Multiple upgrade paths (click power, auto-solver, solve speed)
- **WebAssembly Support**: Runs in the browser with WebGL
- **Real-time Stats**: Track your total solves, solves per second, and current solve progress

## Running Locally (Native)

### Prerequisites
- Rust toolchain (install from [rustup.rs](https://rustup.rs/))

### Run the game
```bash
cargo run --release
```

The game will launch in a native window.

## Building for WebAssembly

### Prerequisites
1. Install the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

2. Install a local web server (choose one):
```bash
# Python 3
python3 -m http.server

# or Node.js
npm install -g http-server

# or Rust
cargo install basic-http-server
```

### Build for WASM
```bash
cargo build --release --target wasm32-unknown-unknown
```

### Copy the WASM file
```bash
cp target/wasm32-unknown-unknown/release/idle_cube.wasm .
```

### Serve the game
```bash
# Using Python
python3 -m http.server 8080

# Using basic-http-server
basic-http-server .

# Using http-server
http-server -p 8080
```

Then open your browser to `http://localhost:8080`

## Quick Build Script

For convenience, you can use this one-liner to build and prepare for web:

```bash
cargo build --release --target wasm32-unknown-unknown && cp target/wasm32-unknown-unknown/release/idle_cube.wasm .
```

## Game Controls

- **Left Click**:
  - Click the cube to add solve progress
  - Click upgrade buttons on the left panel to purchase upgrades

## Upgrade System

1. **Click Power**: Increases progress gained per click
   - Initial cost: 10 solves
   - Cost multiplier: 1.5x

2. **Auto-Solver**: Generates passive solve progress over time
   - Initial cost: 50 solves
   - Cost multiplier: 1.8x
   - Each level adds 1 solve per second (affected by solve speed)

3. **Solve Speed**: Reduces time required per solve
   - Initial cost: 100 solves
   - Cost multiplier: 2.0x
   - Each upgrade reduces solve time by 10% (minimum 0.5s)

## Technical Details

### Shaders
- **Vertex Shader**: Handles 3D cube geometry with normal data for lighting
- **Fragment Shader**: Creates dynamic color effects based on vertex position and time

### Architecture
- Built with [macroquad](https://github.com/not-fl3/macroquad) game framework
- Custom GLSL shaders (version 100 for WebGL compatibility)
- Pure Rust implementation with zero JavaScript dependencies

### Performance
- Optimized for size with `opt-level = "z"` and LTO
- Typical WASM bundle size: ~500KB (optimized)
- Runs at 60 FPS on modern browsers

## Project Structure

```
idle-cube/
├── src/
│   └── main.rs          # Main game logic, shaders, and rendering
├── Cargo.toml           # Rust dependencies and build config
├── index.html           # WebAssembly HTML wrapper
└── README.md            # This file
```

## Future Enhancements

Potential features to add:
- Save/load system (localStorage for web)
- More upgrade types (cube algorithms, different cube sizes)
- Visual upgrades (cube appearance changes with progression)
- Prestige system
- Achievement system
- Sound effects and music
- Mobile touch controls optimization

## License

This is a demo project. Feel free to use and modify as you wish.

## Credits

Built with:
- [macroquad](https://github.com/not-fl3/macroquad) - Rust game framework
- [Rust](https://www.rust-lang.org/) - Programming language
