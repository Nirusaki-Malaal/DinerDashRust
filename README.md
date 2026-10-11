# Diner Dash Rust

A modern, native reimplementation of Diner Dash (2004) written in Rust.
Made by Nirusaki.

This project recreates the original game mechanics, sound, and graphics with high performance. It runs natively on Linux (Wayland and X11), Windows, Android, and Web (WASM).

## Features

- Fast and native performance with no web wrappers for desktop builds.
- Native Wayland and X11 support on Linux with window decorations.
- Native Windows 64-bit support.
- Android APK build with full touchscreen controls and display scaling.
- WebAssembly (WASM) browser version.
- Multi-profile player system with support for up to 10 players.
- Full options menu for music, sound effects, and display modes.
- Original High Scores board with Flo's Career and Endless modes.
- Restored original graphics, animations, fonts, and 44.1 kHz stereo audio.
- Authentic customer patience, seating bonuses, and level progression.

## Requirements

- Rust (edition 2021)
- Cargo
- GCC / MinGW (for Windows cross-compilation)
- Android NDK and SDK (for APK builds)
- wasm-pack (for WASM builds)

## Build Instructions

### Linux Native Binary
```bash
make linux
```
The output binary will be located at `dist/linux/diner-dash`.

### Windows Executable
```bash
make windows
```
The output executable will be located at `dist/windows/diner-dash.exe`.

### Android Package (APK)
```bash
make apk
```
The signed and aligned APK will be located at `dist/android/diner-dash.apk`.

### WebAssembly (WASM)
```bash
make build
```
The web build will be created in `wasm/pkg`. Run locally with:
```bash
make run
```

### Run All Checks and Tests
```bash
make native-check
```

## Automated Builds and Releases

GitHub Actions workflows run automated CI checks and release packaging:
- **CI Workflow (`.github/workflows/ci.yml`)**: Builds and tests on every push and pull request across Linux, Windows, Android, and WebAssembly.
- **Release Workflow (`.github/workflows/release.yml`)**: Triggered on release tags (`v*`) or manual dispatch to build, package, hash (SHA256), and attach binaries to GitHub Releases automatically.

## Controls

- Left Click / Touch: Drag customers, seat tables, take orders, serve food, clear dishes.
- Space / Enter: Interact with kitchen counter.
- Number Keys (1-8): Select table by number.
- P Key: Call podium host.
- F11: Toggle fullscreen mode.
- Escape / Back: Pause game or return to previous screen.

## Project Structure

- `engine/`: Core game logic, state machines, customer AI, and level simulation.
- `native/`: Native desktop and Android frontends using Macroquad.
- `wasm/`: Web frontend compiled with wasm-bindgen.
- `assets/`: Game artwork, sound effects, stage music, and layout definitions.
- `decompiled_lingo/`: Decompiled source scripts from the original 2004 release.
- `docs/`: Technical architecture documentation.

## License

This project is licensed under the GNU General Public License v3.0 (GPL-3.0). See the [LICENSE](LICENSE) file for details.

This reverse-engineered project and its reimplementation were created for preservation and research purposes.
Made by Nirusaki.
