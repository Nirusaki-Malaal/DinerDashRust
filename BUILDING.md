# Native Diner Dash builds

The `native` Rust crate runs the existing game engine through Macroquad's native
OpenGL/OpenGL ES, input, and audio backends. It does not use Electron, Tauri,
WebView, JavaScript, or a browser. Images and audio are embedded in the binaries.
The browser build and its existing Make targets remain available.

## Build commands (Linux x86-64 host)

```sh
make dist       # Build all three platforms
make linux      # dist/linux/diner-dash
make windows    # dist/windows/diner-dash.exe
make apk        # dist/android/diner-dash.apk
make native-check
```

`dist/SHA256SUMS` records output hashes. Desktop executables can be copied and run
without an assets directory. Linux requires the system's graphics driver, X11 or
XWayland, and ALSA. Linux releases built on this host require its glibc version;
use an older Linux build environment when targeting older distributions.
Windows targets 64-bit Windows. The APK includes ARM64 phones/tablets and x86-64
Android devices/emulators, requires Android 8.0/API 26+, and runs in landscape.

## Prerequisites

Install a C compiler, MinGW-w64 (including `x86_64-w64-mingw32-gcc` and `windres`),
ALSA development libraries, Python 3, curl, and a JDK providing `java`, `javac`,
and `keytool`. Install Android command-line tools at `$HOME/Android/Sdk` or set
`ANDROID_HOME`. Accept any required Android SDK licenses using `sdkmanager --licenses`.

```sh
make native-setup
```

Setup installs Rust **1.94.0** and its Windows GNU, Android ARM64, and Android
x86-64 targets in `.toolchains/`, without changing your shell configuration or
system Rust. It installs Android platform/build tools **34/34.0.0** and NDK
**28.2.13676358** through your Android SDK manager. Downloads may be large.
The build script uses this local toolchain when installed, otherwise Cargo on PATH.

Overrides: `ANDROID_HOME`, `ANDROID_NDK_HOME`, `ANDROID_COMPILE_SDK`,
`ANDROID_BUILD_TOOLS`, `ANDROID_ABIS` (space-separated `arm64-v8a x86_64`), and
`CARGO_BUILD_JOBS`. For example:

```sh
ANDROID_ABIS=arm64-v8a make apk
```

The Android script builds the native shared libraries, compiles the Java activity
shipped with the locked Miniquad dependency, creates resources with `aapt`,
compiles Java bytecode with `d8`, aligns the APK, signs it, and verifies the signature.
It requires no Gradle or third-party APK build service. Native ELF segments use
16 KiB alignment; native libraries are compressed and extracted on install.

## APK signing

By default, the script generates a persistent **local development signing key**
at `.build/keys/diner-dash-local.jks` (password `android`, alias `diner-dash`).
The resulting APK is signed and installable for local testing; it is not a Play
Store release. Preserve the key to install later builds as updates.

To use your own existing release key:

```sh
export APK_KEYSTORE=/absolute/path/to/release.jks
export APK_KEY_ALIAS=my-alias
export APK_STORE_PASSWORD='your-store-password'
# Set APK_KEY_PASSWORD too if the key password differs.
make apk
```

Passwords are passed via environment variables, not command-line literals.
Signing keys and generated artifacts are ignored by Git. `make clean` only
cleans the existing web output; it does not remove your signing key.

## Icon, metadata, and controls

`app_icon.ico` matches the original `../Diner Dash.exe` icon resource. Windows
embeds the ICO and version information. Android launcher icons and the native
window icons are derived from it; regenerate with `make icons` (requires Pillow).
The title, in-game credit, Windows version information, Android manifest metadata,
and Linux desktop entry contain **Made by Nirusaki**.

The native menus use the same original title-screen artwork, animation timings,
career map, help pages, comics, pre-level chalkboard, and result artwork as the web
frontend. `scripts/prepare_layouts.py` mirrors the web layout parser; the game
coordinates remain 800 × 600. Menu lettering is the original bitmap artwork.
Dynamic text uses the bundled Noto Sans Bold font, matching this host’s browser
fallback. The Noto font license is included in `packaging/fonts/NOTO-LICENSE`. Drag or tap parties/tables/stations. Keyboard: 1–8 selects tables,
Space/Enter selects the kitchen, P selects the podium, Esc pauses, and F11 toggles
fullscreen. On Android, Back pauses/returns to menus. The original in-canvas Pause button is touch accessible.
Settings contains the “Made by Nirusaki” credit. Export writes `backup.json` next
to the save; Import reads that file (it does not open the browser file picker).

Progress and the current shift are saved every two seconds and at menu transitions.
On desktop, saves live in the user's local data directory under `Nirusaki/DinerDash`;
on Android they live in the app's private files directory. Browser saves are
separate. Returning from the Android background pauses the active shift.

## Smaller Android toolchain option

If you already have host LLVM (`clang`, `ld.lld`, `llvm-ar`, `llvm-strip`),
`make android-linker` downloads approximately 6 MB of the official NDK r28c
headers and API 26 link libraries into `.toolchains/android-ndk-r28c-minimal`.
Each extracted file is checked against its ZIP CRC. It creates compiler wrappers
that use host LLVM with the Android sysroot. The build script automatically uses
this toolchain when the full configured NDK is absent. Rust Android target
libraries and Android SDK platform/build tools are still required. A full NDK
can always be selected with `ANDROID_NDK_HOME`.
