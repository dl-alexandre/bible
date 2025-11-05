# Bible Datasets

This directory contains public domain English Bible translations in the standardized format used by the Bible Static Generator.

## Format

All Bible files follow this format:
- Book name on its own line (e.g., "Genesis")
- Chapter header: "Chapter X"
- Numbered verses: "N Verse text..."
- Blank line between chapters

## Available Translations

### KJV (King James Version)
- **File**: `kjv.txt`
- **Status**: ✓ Complete
- **Source**: Project Gutenberg (public domain)
- **Lines**: 33,085

### ASV (American Standard Version, 1901)
- **File**: `asv.txt`
- **Status**: ✓ Complete
- **Source**: Project Gutenberg (public domain)
- **Lines**: 33,084

## Usage

The Bible Static Generator reads these files via the `--datasets` flag:

```bash
cargo build --release
./target/release/bible-static-generator \
  --datasets datasets/kjv.txt datasets/asv.txt \
  --out out
```

The version code is automatically derived from the filename (e.g., `kjv.txt` → `kjv`).

## Adding More Translations

To add additional translations:

1. Download the source text (must be public domain)
2. Convert to the standardized format using `convert_to_kjv_format.py`
3. Save as `{version}.txt` in this directory
4. Update the `BibleFormat` enum in `src/parser.rs` if needed

## Future Additions

- **WEB (World English Bible)**: Public domain, needs format conversion
- **OEB (Open English Bible)**: Public domain, needs format conversion
- **YLT (Young's Literal Translation)**: Public domain, needs source and conversion

