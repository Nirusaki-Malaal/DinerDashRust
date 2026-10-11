# Diner Dash Rust

A modern, native reimplementation of Diner Dash (2004) written in Rust.
Made by Nirusaki.

This project recreates the original game mechanics, sound, and graphics with high performance. It runs natively on Linux (Wayland and X11), Windows, macOS (Apple Silicon), Android, and Web (WASM).

## Features

- Fast and native performance with no web wrappers for desktop builds.
- Native Wayland and X11 support on Linux with window decorations.
- Native Windows 64-bit support.
- Native macOS Apple Silicon (ARM64) support with Cocoa and Metal.
- Android APK build with full touchscreen controls and display scaling.
- WebAssembly (WASM) browser version.
- Multi-profile player system with support for up to 10 players.
- Full options menu for music, sound effects, and display modes.
- Original High Scores board with Flo's Career and Endless modes.
- Restored original graphics, animations, fonts, and 44.1 kHz stereo audio.
- Authentic customer patience, seating bonuses, and level progression.

## Why This Version Is Better

The original 2004 game was built as a 32-bit Macromedia Director projector with many limitations on modern computers. This Rust reimplementation improves upon the old binaries:

- **Runs Everywhere**: The old game only ran on legacy 32-bit Windows. This version runs natively on 64-bit Linux (Wayland and X11), modern Windows, macOS (Apple Silicon), Android, and WebAssembly in browsers.
- **Modern Display Scaling**: The old binaries were locked to a fixed 800x600 window. This version supports fullscreen scaling, high-DPI monitors, and wide screen display on Android.
- **Touchscreen and Mobile Ready**: The original required mouse input. This version includes full touchscreen drag-and-drop controls for phones and tablets.
- **Audio Quality**: Replaced legacy 22 kHz mono audio with clean 44.1 kHz stereo sound and accurate music looping.
- **Crash Resistance and Safe Saves**: The original Director engine suffered from memory leaks and corrupted save files on sudden exits. This version uses memory-safe Rust and atomic JSON saves that prevent data corruption.
- **No Emulators or Compatibility Layers**: Runs as a direct native binary with zero legacy dependencies like DirectDraw wrappers or virtual machines.

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

### macOS (Apple Silicon) Binary
```bash
make macos
```
The output binary will be located at `dist/macos/diner-dash`.

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
- **CI Workflow (`.github/workflows/ci.yml`)**: Builds and tests on every push and pull request across Linux, Windows, macOS, Android, and WebAssembly.
- **Release Workflow (`.github/workflows/release.yml`)**: Triggered on release tags (`v*`) or manual dispatch to build, package, hash (SHA256), and attach binaries to GitHub Releases automatically for Linux, Windows, macOS, Android, and Web.

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

## Credits

- **Original Game**: Created by Gamelab and published by PlayFirst (2004). All copyrights and trademarks are reserved to the original creators.
- **Reimplementation and Reverse Engineering**: Made by Nirusaki.
- **Decompiler Tooling**: ProjectorRays team for Shockwave movie research.

## AI Disclosure

Local open models and open-weight tools were used to accelerate this project. They assisted in parsing raw decompiler outputs, analyzing legacy Lingo bytecode, and translating math logic into Rust. Without this acceleration, completing this reverse engineering and rewrite would have taken over six months.

## License

This project is licensed under the GNU General Public License v3.0 (GPL-3.0). See the [LICENSE](LICENSE) file for details.

This reverse-engineered project and its reimplementation were created for preservation and research purposes. All copyrights and trademarks are reserved to the original creators.
Made by Nirusaki.
