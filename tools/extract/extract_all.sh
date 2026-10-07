#!/bin/bash
# Batch extraction script for Diner Dash assets
# Run from the diner_dash_wasm directory

set -e

# Configuration
GAME_DIR="/home/nirusaki/Diner_Dash_Win_Files_EN/Game Files"
EXTRACT_DIR="/tmp/diner_dash_extracted"
CHUNKS_DIR="/tmp/diner_dash_chunks"
OUTPUT_DIR="./assets"
PROJECTORRAYS="/tmp/ProjectorRays/projectorrays"

echo "=========================================="
echo "Diner Dash Asset Extraction"
echo "=========================================="

# Check ProjectorRays exists
if [ ! -f "$PROJECTORRAYS" ]; then
    echo "ERROR: ProjectorRays not found at $PROJECTORRAYS"
    echo "Build it first: cd /tmp/ProjectorRays && make release"
    exit 1
fi

# Create output directories
mkdir -p "$OUTPUT_DIR/images"
mkdir -p "$OUTPUT_DIR/audio"
mkdir -p "$OUTPUT_DIR/data"

echo ""
echo "Step 1: Decompile all casts with ProjectorRays (dump chunks)..."
echo "------------------------------------------"

# Decompile graphics casts
echo "Decompiling graphics..."
$PROJECTORRAYS decompile "$GAME_DIR/gfx" -o "$CHUNKS_DIR" --dump-chunks 2>&1 | grep -E "(Decompiled|Error)"

# Decompile sound casts
echo "Decompiling sounds..."
$PROJECTORRAYS decompile "$GAME_DIR/snd" -o "$CHUNKS_DIR" --dump-chunks 2>&1 | grep -E "(Decompiled|Error)"

# Decompile config
echo "Decompiling config..."
$PROJECTORRAYS decompile "$GAME_DIR/config" -o "$CHUNKS_DIR" --dump-chunks 2>&1 | grep -E "(Decompiled|Error)"

# Decompile ext casts
echo "Decompiling ext casts..."
$PROJECTORRAYS decompile "$GAME_DIR/ext" -o "$EXTRACT_DIR/ext" --dump-chunks 2>&1 | grep -E "(Decompiled|Error)"

# Decompile splash
echo "Decompiling splash..."
$PROJECTORRAYS decompile "$GAME_DIR/gamelab_splash.dcr" -o "$EXTRACT_DIR" --dump-chunks 2>&1 | grep -E "(Decompiled|Error)"

echo ""
echo "Step 2: Extract raw chunk data..."
echo "------------------------------------------"

# Copy BITD chunks
echo "Copying BITD chunks..."
find "$CHUNKS_DIR" -name "BITD-*.bin" -exec cp {} "$OUTPUT_DIR/images/" \;
find "$EXTRACT_DIR" -name "BITD-*.bin" -exec cp {} "$OUTPUT_DIR/images/" \;

# Copy ediM chunks
echo "Copying ediM chunks..."
find "$CHUNKS_DIR" -name "ediM-*.bin" -exec cp {} "$OUTPUT_DIR/audio/" \;
find "$EXTRACT_DIR" -name "ediM-*.bin" -exec cp {} "$OUTPUT_DIR/audio/" \;

echo ""
echo "Step 3: Generate metadata from cast info..."
echo "------------------------------------------"

# Run metadata extraction
cargo run --release -p diner-dash-extract -- extract \
    --input "$EXTRACT_DIR" \
    --output "$OUTPUT_DIR/data" 2>&1 | tail -20

echo ""
echo "Step 4: Decode assets using metadata..."
echo "------------------------------------------"

# This would be a Rust program that reads the metadata and decodes
# For now, just count what we have
BITD_COUNT=$(find "$OUTPUT_DIR/images" -name "BITD-*.bin" | wc -l)
EDIM_COUNT=$(find "$OUTPUT_DIR/audio" -name "ediM-*.bin" | wc -l)

echo "Found $BITD_COUNT BITD chunks"
echo "Found $EDIM_COUNT ediM chunks"

echo ""
echo "Step 5: Create asset manifest..."
echo "------------------------------------------"

# Generate manifest (will be done by Rust tool)
cargo run --release -p diner-dash-extract -- manifest \
    --input "$OUTPUT_DIR" \
    --output "$OUTPUT_DIR/data/manifest.ron" 2>&1 || true

echo ""
echo "=========================================="
echo "Extraction complete!"
echo "Assets in: $OUTPUT_DIR"
echo "=========================================="