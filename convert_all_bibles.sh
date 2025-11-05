#!/bin/bash

set -e

DATASETS_DIR="datasets"
CONVERTER="convert_to_kjv_format.py"

echo "Converting all downloaded Bible sources to kjv.txt format..."
echo ""

convert_bible() {
    local source_file=$1
    local output_file=$2
    local version_name=$3
    
    if [ ! -f "$source_file" ]; then
        echo "✗ $version_name: Source file not found ($source_file)"
        return 1
    fi
    
    if [ ! -s "$source_file" ]; then
        echo "✗ $version_name: Source file is empty"
        return 1
    fi
    
    local lines=$(wc -l < "$source_file" | tr -d ' ')
    if [ "$lines" -lt 100 ]; then
        echo "✗ $version_name: Source file too small ($lines lines), skipping"
        return 1
    fi
    
    echo "Converting $version_name ($lines source lines)..."
    python3 "$CONVERTER" "$source_file" "$output_file" 2>&1 | tail -1
    
    if [ -f "$output_file" ] && [ -s "$output_file" ]; then
        output_lines=$(wc -l < "$output_file" | tr -d ' ')
        if [ "$output_lines" -gt 1000 ]; then
            if ! head -1 "$output_file" | grep -q "^Genesis$"; then
                sed -i '' '1i\
Genesis
' "$output_file" 2>/dev/null || sed -i '1i\
Genesis
' "$output_file"
            fi
            echo "✓ $version_name: Converted successfully ($output_lines lines)"
            return 0
        else
            echo "✗ $version_name: Conversion produced too few lines ($output_lines), may need different converter"
            rm -f "$output_file"
            return 1
        fi
    else
        echo "✗ $version_name: Conversion failed"
        return 1
    fi
}

echo "Converting Bibles:"
echo "=================="
echo ""

convert_bible "$DATASETS_DIR/kjv_source.txt" "$DATASETS_DIR/kjv.txt" "KJV"
convert_bible "$DATASETS_DIR/asv_source.txt" "$DATASETS_DIR/asv.txt" "ASV"

if [ -f "$DATASETS_DIR/web_source.txt" ]; then
    convert_bible "$DATASETS_DIR/web_source.txt" "$DATASETS_DIR/web.txt" "WEB"
fi

if [ -f "$DATASETS_DIR/oeb_source.txt" ]; then
    convert_bible "$DATASETS_DIR/oeb_source.txt" "$DATASETS_DIR/oeb.txt" "OEB"
fi

echo ""
echo "Conversion summary:"
echo "==================="
ls -lh "$DATASETS_DIR"/*.txt 2>/dev/null | grep -v "_source" | grep -v "README" | awk '{print $9, "(" $5 ", " $6 ")"}'
echo ""
echo "Ready Bibles:"
for file in "$DATASETS_DIR"/*.txt; do
    if [ -f "$file" ] && [[ ! "$(basename "$file")" =~ (source|README) ]]; then
        lines=$(wc -l < "$file" | tr -d ' ')
        if [ "$lines" -gt 1000 ]; then
            echo "  ✓ $(basename "$file" .txt) - $lines lines"
        fi
    fi
done

