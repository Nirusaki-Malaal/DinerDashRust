#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
ROOT_DIR="$PWD"
export RUSTUP_HOME="$ROOT_DIR/.toolchains/rustup"
export CARGO_HOME="$ROOT_DIR/.toolchains/cargo"
export PATH="$CARGO_HOME/bin:$PATH"
mkdir -p "$ROOT_DIR/.build"
if [[ ! -x "$CARGO_HOME/bin/rustup" ]]; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$ROOT_DIR/.build/rustup-init.sh"
    RUSTUP_INIT_SKIP_PATH_CHECK=yes sh "$ROOT_DIR/.build/rustup-init.sh" -y --no-modify-path --profile minimal --default-toolchain 1.94.0
fi
rustup toolchain install 1.94.0 --profile minimal
rustup target add --toolchain 1.94.0 x86_64-pc-windows-gnu aarch64-linux-android x86_64-linux-android
SDK_DIR="${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}"
SDK_MANAGER="$SDK_DIR/cmdline-tools/latest/bin/sdkmanager"
if [[ ! -x "$SDK_MANAGER" ]]; then
    echo "Install Android command-line tools at $SDK_DIR; see README.md." >&2
    exit 1
fi
"$SDK_MANAGER" --sdk_root="$SDK_DIR" 'platforms;android-34' 'build-tools;34.0.0' 'ndk;28.2.13676358'
echo 'Ready: make dist (requires JDK, MinGW-w64, and Linux ALSA development files).'
