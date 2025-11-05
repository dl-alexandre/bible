#!/bin/bash

set -e

echo "Downloading Kaggle dataset: 29-public-domain-bibles-with-verse-character-count"
echo ""

if [ ! -f ~/.kaggle/kaggle.json ]; then
    echo "ERROR: Kaggle credentials not found!"
    echo ""
    echo "Please set up your Kaggle API credentials:"
    echo "1. Go to https://www.kaggle.com/settings"
    echo "2. Scroll to 'API' section and click 'Create New Token'"
    echo "3. Move the downloaded kaggle.json file to ~/.kaggle/kaggle.json"
    echo "4. Run: chmod 600 ~/.kaggle/kaggle.json"
    echo ""
    exit 1
fi

chmod 600 ~/.kaggle/kaggle.json

DATASET="dfydata/29-public-domain-bibles-with-verse-character-count"
OUTPUT_DIR="datasets/29-public-domain-bibles"

echo "Downloading dataset to: $OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

python3 -m kaggle datasets download -d "$DATASET" -p "$OUTPUT_DIR" --unzip

echo ""
echo "Dataset downloaded successfully to $OUTPUT_DIR"
echo "Listing contents:"
ls -lh "$OUTPUT_DIR"


