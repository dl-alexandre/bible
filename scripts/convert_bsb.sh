#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

DATASETS_DIR="datasets"
BSB_USFM_DIR="$DATASETS_DIR/bsb_usfm"
BSB_OUTPUT="$DATASETS_DIR/bsb.txt"

if [ ! -d "$BSB_USFM_DIR" ]; then
    echo "Error: BSB USFM directory not found: $BSB_USFM_DIR"
    echo ""
    echo "Please download BSB USFM files from:"
    echo "  https://berean.bible/downloads.htm"
    echo ""
    echo "Then extract/save the USFM files to: $BSB_USFM_DIR/"
    exit 1
fi

if [ ! "$(ls -A $BSB_USFM_DIR/*.{usfm,SFM,sfm} 2>/dev/null)" ]; then
    echo "Error: No USFM files (.usfm, .SFM, or .sfm) found in $BSB_USFM_DIR/"
    echo ""
    echo "Please download BSB USFM files from:"
    echo "  https://berean.bible/downloads.htm"
    echo ""
    echo "Then extract/save the USFM files to: $BSB_USFM_DIR/"
    exit 1
fi

echo "Converting BSB USFM files to standardized format..."
echo "Input directory: $BSB_USFM_DIR"
echo "Output file: $BSB_OUTPUT"
echo ""

python3 "$SCRIPT_DIR/convert_usfm_to_kjv_format.py" "$BSB_USFM_DIR" "$BSB_OUTPUT"

if [ -f "$BSB_OUTPUT" ] && [ -s "$BSB_OUTPUT" ]; then
    LINE_COUNT=$(wc -l < "$BSB_OUTPUT" | tr -d ' ')
    echo ""
    echo "✓ BSB conversion complete!"
    echo "  Output: $BSB_OUTPUT"
    echo "  Lines: $LINE_COUNT"
else
    echo ""
    echo "✗ Conversion failed or produced empty file"
    exit 1
fi

