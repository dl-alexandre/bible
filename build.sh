#!/bin/bash

set -e

OUT_DIR="${OUT_DIR:-out}"
BASE_URL="${BASE_URL:-https://dl-alexandre.github.io/Bible/}"

echo "Building Bible Static Site..."
echo "Output directory: $OUT_DIR"
echo "Base URL: $BASE_URL"
echo ""

cargo build --release

echo ""
echo "Generating site..."

./target/release/bible-static-generator \
  --datasets datasets/kjv.txt \
  --datasets datasets/asv.txt \
  --datasets datasets/web.txt \
  --out "$OUT_DIR" \
  --base-url "$BASE_URL" \
  --minify-json \
  --gzip-json \
  --utc-timestamp \
  --threshold-jaccard 0.70 \
  --threshold-lev 0.15

echo ""
echo "Removing JSON chapter files (HTML-only mode)..."
rm -rf "$OUT_DIR/kjv" "$OUT_DIR/asv" "$OUT_DIR/web"

echo ""
echo "Build complete! Site ready in $OUT_DIR/"
echo "HTML files: $(find "$OUT_DIR/bible" -name '*.html' | wc -l | tr -d ' ')"

