#!/bin/bash

set -e

OUT_DIR="${OUT_DIR:-out}"
BASE_URL="${BASE_URL:-https://dl-alexandre.github.io/bible/}"

echo "Building Bible Static Site..."
echo "Output directory: $OUT_DIR"
echo "Base URL: $BASE_URL"
echo ""

cargo build --release

echo ""
echo "Setting up static assets..."
mkdir -p "$OUT_DIR/static"

if [ -f "static/og-image.png" ]; then
    cp static/og-image.png "$OUT_DIR/static/" && echo "✅ Copied og-image.png"
else
    echo "⚠️  Warning: static/og-image.png not found"
fi

if [ -f "static/favicon.ico" ]; then
    cp static/favicon.ico "$OUT_DIR/static/" && echo "✅ Copied favicon.ico"
else
    echo "Generating favicon..."
    python3 scripts/create_favicon.py || echo "⚠️  Warning: Could not generate favicon"
fi

if [ -f "static/styles.css" ]; then
    cp static/styles.css "$OUT_DIR/static/" && echo "✅ Copied styles.css"
else
    echo "⚠️  Warning: static/styles.css not found"
fi

if [ -f "static/app.webmanifest" ]; then
    cp static/app.webmanifest "$OUT_DIR/static/" && echo "✅ Copied app.webmanifest"
fi
if [ -f "static/verses.js" ]; then
    cp static/verses.js "$OUT_DIR/static/" && echo "✅ Copied verses.js"
fi
if [ -f "static/sw.js" ]; then
    cp static/sw.js "$OUT_DIR/sw.js" && echo "✅ Copied service worker"
fi

echo ""
echo "Generating site..."

./target/release/bible-static-generator \
  --datasets datasets/kjv.txt \
  --datasets datasets/asv.txt \
  --datasets datasets/web.txt \
  --datasets datasets/bsb.txt \
  --out "$OUT_DIR" \
  --base-url "$BASE_URL" \
  --minify-json \
  --gzip-json \
  --utc-timestamp \
  --threshold-jaccard 0.70 \
  --threshold-lev 0.15

echo ""
echo "Merging JSON API files into the site output..."
for version in kjv asv web bsb; do
    cp -R "$OUT_DIR/$version/." "$OUT_DIR/bible/$version/"
    rm -rf "$OUT_DIR/$version"
done
if [ -f "$OUT_DIR/sw.js" ]; then
    mv "$OUT_DIR/sw.js" "$OUT_DIR/bible/sw.js"
fi
if [ -d "$OUT_DIR/static" ]; then
    mv "$OUT_DIR/static" "$OUT_DIR/bible/static"
fi

echo ""
echo "Build complete! Site ready in $OUT_DIR/"
echo "HTML files: $(find "$OUT_DIR/bible" -name '*.html' | wc -l | tr -d ' ')"
echo ""
echo "Static site ready for deployment."
