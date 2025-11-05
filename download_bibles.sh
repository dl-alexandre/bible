#!/bin/bash

set -e

DATASETS_DIR="datasets"
mkdir -p "$DATASETS_DIR"

echo "Downloading public domain English Bibles..."
echo ""

download_kjv() {
    if [ -f "$DATASETS_DIR/kjv.txt" ]; then
        echo "✓ KJV already exists"
        return
    fi
    echo "Downloading KJV..."
    curl -s "https://www.gutenberg.org/files/10/10-0.txt" -o "$DATASETS_DIR/kjv_source.txt"
    echo "✓ KJV source downloaded (needs conversion)"
}

download_asv() {
    if [ -f "$DATASETS_DIR/asv.txt" ]; then
        echo "✓ ASV already exists"
        return
    fi
    echo "Downloading ASV (American Standard Version, 1901)..."
    curl -s "https://www.gutenberg.org/files/10/10-0.txt" -o "$DATASETS_DIR/asv_source.txt"
    if [ -s "$DATASETS_DIR/asv_source.txt" ]; then
        echo "✓ ASV source downloaded (needs conversion)"
    else
        echo "✗ ASV download failed"
    fi
}

download_web() {
    if [ -f "$DATASETS_DIR/web.txt" ]; then
        echo "✓ WEB already exists"
        return
    fi
    echo "Downloading WEB (World English Bible)..."
    curl -sL "https://raw.githubusercontent.com/world-english-bible/web/main/WEB.txt" -o "$DATASETS_DIR/web_raw.txt" || {
        echo "Trying alternative source..."
        curl -sL "https://api.bible/bibles/de4e12af7f28f599-02/books" -o /dev/null || echo "API unavailable"
    }
    if [ -s "$DATASETS_DIR/web_raw.txt" ]; then
        echo "✓ WEB downloaded (needs conversion)"
    else
        echo "✗ WEB download failed - may need manual download"
    fi
}

download_oeb() {
    if [ -f "$DATASETS_DIR/oeb.txt" ]; then
        echo "✓ OEB already exists"
        return
    fi
    echo "Downloading OEB (Open English Bible)..."
    curl -sL "https://openenglishbible.org/oeb/2021.1/oeb.txt" -o "$DATASETS_DIR/oeb_raw.txt" || {
        echo "Trying alternative..."
        curl -sL "https://raw.githubusercontent.com/openenglishbible/Open-English-Bible/master/oeb.txt" -o "$DATASETS_DIR/oeb_raw.txt" || echo "✗ OEB download failed"
    }
    if [ -s "$DATASETS_DIR/oeb_raw.txt" ]; then
        echo "✓ OEB downloaded (needs conversion)"
    fi
}

download_ylt() {
    if [ -f "$DATASETS_DIR/ylt.txt" ]; then
        echo "✓ YLT already exists"
        return
    fi
    echo "Downloading YLT (Young's Literal Translation)..."
    curl -s "https://www.gutenberg.org/files/8294/8294-0.txt" -o "$DATASETS_DIR/ylt_source.txt" || {
        echo "✗ YLT download failed"
        return
    }
    if [ -s "$DATASETS_DIR/ylt_source.txt" ]; then
        echo "✓ YLT source downloaded (needs conversion)"
    fi
}

echo "Downloading Bibles..."
download_kjv
download_asv
download_web
download_oeb
download_ylt

echo ""
echo "Downloads complete. Files in $DATASETS_DIR/:"
ls -lh "$DATASETS_DIR/" | grep -E "\.(txt|source|raw)" || ls -lh "$DATASETS_DIR/"
echo ""
echo "Note: Files ending in _source.txt or _raw.txt need format conversion"
echo "to match the kjv.txt format (book name, Chapter X, numbered verses)"
