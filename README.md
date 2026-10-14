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

## Running in the browser

Install the target and build the static site:

```bash
rustup target add wasm32-unknown-unknown
./build_wasm.sh
python3 -m http.server 8080 --directory dist
```

Open `http://localhost:8080`. Serve over HTTP rather than opening `index.html`
directly. The build packages `index.html`, `idle_cube.wasm`, and
`mq_js_bundle.js` into `dist/`; shaders are embedded in the WASM binary.

The JavaScript loader in `web/mq_js_bundle.js` comes from the **macroquad 0.4.14**
crate (`js/mq_js_bundle.js`) and includes its JavaScript plugins. When upgrading
macroquad, replace this bundle with the one from the new crate and verify browser
startup. A local fix ensures matrix arrays upload all their elements (see
`web/README.md`). The older miniquad `gl-0.2.8.js` loader is not used. Scrambles use a
clock-seeded PRNG so the binary requires no wasm-bindgen JavaScript glue.

## Deploying to GitHub Pages

1. Push this repository and the workflow to GitHub on the `main` branch.
2. In **Settings → Pages → Build and deployment**, select **GitHub Actions**
   as the source.
3. Push to `main` or run **Build and deploy browser game** from the Actions tab.

The workflow tests the game, builds WASM, and publishes only `dist/`. Pull
requests run the tests and build without deploying. The deployment URL appears
in the workflow's `github-pages` environment, normally
`https://<owner>.github.io/<repository>/`.

All asset URLs are relative, so repository subpaths and custom domains work
without changing the build. To host elsewhere, upload the contents of `dist/`
to any static web host. The checked-in root `idle_cube.wasm` is an old artifact;
the build and deployment always use the newly compiled binary in `dist/`.

## Progression and controls

Solves run automatically. Each solve earns XP, scaled by its time and your
unspent Cube Knowledge bonus. Leveling grants one skill point and increases
maximum TPS. Click technique buttons to spend skill points; Direct Cross is
the prerequisite for Intuitive F2L and individual last-layer algorithms.

### Prestige and Cube Knowledge

Prestige resets levels, run XP, skill points, learned algorithms, proficiency,
and solve statistics. Permanent technique access and stat upgrades survive.
The reset button appears when you can earn Knowledge; click it twice to confirm.

Rewards are `10 × floor(sqrt(run XP / 100))`: 100 XP earns 10 Knowledge,
400 earns 20, and 900 earns 30. Unspent Knowledge grants an XP multiplier of
`1.08 ^ Knowledge`. Spending Knowledge reduces that bonus.

After the first prestige, you can spend **5 Cube Knowledge on permanent
2-look PLL access**. This leaves 5 for about **1.47× XP** or permanent stat
upgrades. All purchases are optional. If you spend below the PLL cost, keep
playing and earn more Knowledge on a later prestige.

**2-look OLL access costs 25 Knowledge** and requires PLL access. Saving the
remaining 5 lets a 400-XP second run fund OLL. Spending it means a longer run or
additional prestiges. Access opens the algorithm branch; individual algorithms
still cost 1 skill point and must be relearned each run.

Permanent buyables start at 2 Knowledge and increase independently to 4, 6,
8, etc. Each purchase raises the TPS cap by 10% or lowers the recognition floor
by 10%. These change training limits; they do not instantly grant mastery.

Saving is not yet implemented; restarting the game starts fresh.

## Technical Details

### Shaders
- **Vertex Shader**: Handles 3D cube geometry with normal data for lighting
- **Fragment Shader**: Creates dynamic color effects based on vertex position and time

### Architecture
- Built with [macroquad](https://github.com/not-fl3/macroquad) game framework
- Custom GLSL shaders (version 100 for WebGL compatibility)
- Rust gameplay with the bundled macroquad JavaScript loader

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
