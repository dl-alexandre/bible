#!/bin/bash

set -e

DATASETS_DIR="datasets"
mkdir -p "$DATASETS_DIR"

echo "Downloading all available public domain English Bibles from Project Gutenberg..."
echo ""

download_gutenberg_bible() {
    local gutenberg_id=$1
    local version_name=$2
    local output_file="$DATASETS_DIR/${version_name}_source.txt"
    
    echo "Downloading $version_name (Gutenberg ID: $gutenberg_id)..."
    curl -sL "https://www.gutenberg.org/files/${gutenberg_id}/${gutenberg_id}-0.txt" -o "$output_file"
    
    if [ -s "$output_file" ]; then
        echo "✓ $version_name source downloaded ($(wc -l < "$output_file" | tr -d ' ') lines)"
        return 0
    else
        echo "✗ $version_name download failed"
        rm -f "$output_file"
        return 1
    fi
}

echo "Downloading Bibles from Project Gutenberg:"
echo ""

download_gutenberg_bible 10 "kjv"
download_gutenberg_bible 10 "asv"
download_gutenberg_bible 8294 "ylt"
download_gutenberg_bible 8295 "darby"

echo ""
echo "Checking for other public domain sources..."
echo ""

download_web() {
    echo "Attempting to download WEB (World English Bible)..."
    curl -sL "https://raw.githubusercontent.com/world-english-bible/web/main/WEB.txt" -o "$DATASETS_DIR/web_source.txt" || {
        curl -sL "https://api.bible/bibles/de4e12af7f28f599-02/books" -o /dev/null 2>&1 || echo "WEB: Alternative sources needed"
        return 1
    }
    if [ -s "$DATASETS_DIR/web_source.txt" ]; then
        echo "✓ WEB downloaded"
        return 0
    fi
    rm -f "$DATASETS_DIR/web_source.txt"
    return 1
}

download_oeb() {
    echo "Attempting to download OEB (Open English Bible)..."
    curl -sL "https://openenglishbible.org/oeb/2021.1/oeb.txt" -o "$DATASETS_DIR/oeb_source.txt" || {
        curl -sL "https://raw.githubusercontent.com/openenglishbible/Open-English-Bible/master/oeb.txt" -o "$DATASETS_DIR/oeb_source.txt" || {
            echo "OEB: Alternative sources needed"
            return 1
        }
    }
    if [ -s "$DATASETS_DIR/oeb_source.txt" ]; then
        echo "✓ OEB downloaded"
        return 0
    fi
    rm -f "$DATASETS_DIR/oeb_source.txt"
    return 1
}

download_web
download_oeb

echo ""
echo "Download summary:"
echo "=================="
ls -lh "$DATASETS_DIR"/*_source.txt 2>/dev/null | awk '{print $9, "(" $5 ")"}' || echo "No source files downloaded"
echo ""
echo "Next step: Run convert_all_bibles.sh to convert to kjv.txt format"

