#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

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

download_bsb() {
    if [ -f "$DATASETS_DIR/bsb.txt" ]; then
        echo "✓ BSB already exists"
        return
    fi
    
    BSB_USFM_DIR="$DATASETS_DIR/bsb_usfm"
    BSB_ZIP="$DATASETS_DIR/bsb_usfm.zip"
    mkdir -p "$BSB_USFM_DIR"
    
    echo "Downloading BSB (Berean Standard Bible) USFM files..."
    
    if [ -d "$BSB_USFM_DIR" ] && [ "$(ls -A $BSB_USFM_DIR/*.{usfm,SFM,sfm} 2>/dev/null)" ]; then
        echo "✓ BSB USFM files already exist in $BSB_USFM_DIR"
        return
    fi
    
    if [ -f "$BSB_ZIP" ]; then
        echo "✓ BSB zip file already exists, extracting..."
        TEMP_EXTRACT="$DATASETS_DIR/bsb_usfm_temp"
        unzip -q -o "$BSB_ZIP" -d "$TEMP_EXTRACT" 2>/dev/null || {
            echo "✗ Failed to extract zip file"
            return
        }
        if [ -d "$TEMP_EXTRACT/bsb_usfm" ]; then
            mv "$TEMP_EXTRACT/bsb_usfm"/* "$BSB_USFM_DIR/" 2>/dev/null || cp "$TEMP_EXTRACT/bsb_usfm"/* "$BSB_USFM_DIR/" 2>/dev/null
            rm -rf "$TEMP_EXTRACT"
        fi
        if [ "$(ls -A $BSB_USFM_DIR/*.{usfm,SFM,sfm} 2>/dev/null)" ]; then
            echo "✓ BSB USFM files extracted successfully"
            return
        fi
    fi
    
    echo "Downloading BSB USFM zip from official source..."
    curl -sL "https://bereanbible.com/bsb_usfm.zip" -o "$BSB_ZIP" || {
        echo "✗ Download failed"
        echo "  Please manually download from: https://berean.bible/downloads.htm"
        echo "  Save as: $BSB_ZIP"
        return
    }
    
    if [ ! -s "$BSB_ZIP" ]; then
        echo "✗ Downloaded file is empty"
        rm -f "$BSB_ZIP"
        return
    fi
    
    echo "✓ BSB zip downloaded ($(du -h "$BSB_ZIP" | cut -f1))"
    echo "Extracting USFM files..."
    
    TEMP_EXTRACT="$DATASETS_DIR/bsb_usfm_temp"
    rm -rf "$TEMP_EXTRACT"
    unzip -q -o "$BSB_ZIP" -d "$TEMP_EXTRACT" 2>/dev/null || {
        echo "✗ Failed to extract zip file"
        rm -rf "$TEMP_EXTRACT"
        return
    }
    
    if [ -d "$TEMP_EXTRACT/bsb_usfm" ]; then
        mv "$TEMP_EXTRACT/bsb_usfm"/* "$BSB_USFM_DIR/" 2>/dev/null || cp "$TEMP_EXTRACT/bsb_usfm"/* "$BSB_USFM_DIR/" 2>/dev/null
        rm -rf "$TEMP_EXTRACT"
    fi
    
    if [ "$(ls -A $BSB_USFM_DIR/*.{usfm,SFM,sfm} 2>/dev/null)" ]; then
        USFM_COUNT=$(ls -1 $BSB_USFM_DIR/*.{usfm,SFM,sfm} 2>/dev/null | wc -l | tr -d ' ')
        echo "✓ BSB USFM files extracted successfully ($USFM_COUNT files)"
        rm -f "$BSB_ZIP"
    else
        echo "✗ No USFM files found in zip"
        rm -rf "$TEMP_EXTRACT"
    fi
}

echo "Downloading Bibles..."
download_kjv
download_asv
download_web
download_oeb
download_ylt
download_bsb

echo ""
echo "Downloads complete. Files in $DATASETS_DIR/:"
ls -lh "$DATASETS_DIR/" | grep -E "\.(txt|source|raw)" || ls -lh "$DATASETS_DIR/"
echo ""
echo "Note: Files ending in _source.txt or _raw.txt need format conversion"
echo "to match the kjv.txt format (book name, Chapter X, numbered verses)"
