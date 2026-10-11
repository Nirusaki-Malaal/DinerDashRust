.DEFAULT_GOAL := run

WASM_DIR := wasm
PORT ?= 8080
PYTHON ?= python3

.PHONY: help build check-tools serve run clean

help:
	@echo "Available targets:"
	@echo "  make dist    - build native Linux, Windows and Android releases"
	@echo "  make linux   - build dist/linux/diner-dash"
	@echo "  make windows - build dist/windows/diner-dash.exe"
	@echo "  make apk     - build signed dist/android/diner-dash.apk"
	@echo "  make native-setup - install project-local Rust cross targets and Android NDK"
	@echo "  make icons   - regenerate platform icons from app_icon.ico (Pillow)"
	@echo "  make native-check - check native frontend and run engine tests"
	@echo "  make build   - build the wasm package"
	@echo "  make run     - build and serve the game locally"
	@echo "  make serve   - serve the built game without rebuilding"
	@echo "  make clean   - remove the generated wasm pkg output"

check-tools:
	@command -v $(PYTHON) >/dev/null 2>&1 || { echo "Python 3 is required but not installed."; exit 1; }
	@command -v wasm-pack >/dev/null 2>&1 || { echo "wasm-pack is required. Install it with: cargo install wasm-pack"; exit 1; }

build: check-tools
	@cd $(WASM_DIR) && wasm-pack build --target web --out-dir pkg --release

serve:
	@$(PYTHON) -m http.server $(PORT) --directory $(WASM_DIR)

run: build
	@echo "Serving Diner Dash at http://localhost:$(PORT)"
	@$(PYTHON) -m http.server $(PORT) --directory $(WASM_DIR)

clean:
	@rm -rf $(WASM_DIR)/pkg

# Native Rust builds (no browser runtime). Outputs are written to dist/.
.PHONY: all native linux windows macos apk android dist icons native-check native-setup

native linux:
	@$(PYTHON) scripts/native_build.py linux

windows:
	@$(PYTHON) scripts/native_build.py windows

macos:
	@$(PYTHON) scripts/native_build.py macos

apk android:
	@$(PYTHON) scripts/native_build.py android

all dist:
	@$(PYTHON) scripts/native_build.py all

icons:
	@$(PYTHON) scripts/prepare_icons.py

native-check:
	@cargo check --locked -p diner-dash-native
	@cargo test --locked -p diner-dash-engine -p diner-dash-native --lib

native-setup:
	@bash scripts/setup_native.sh

.PHONY: android-linker
android-linker:
	@$(PYTHON) scripts/setup_android_linker.py
