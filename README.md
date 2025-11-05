# Bible Static Generator

A deterministic static site generator for Bible datasets, producing HTML and JSON outputs with cross-version mapping, schema validation, and comprehensive build verification.

## Features

- **Deterministic Builds**: Reproducible output with normalized timestamps and sorted structures
- **Cross-Version Mapping**: Intelligent alignment between Bible versions with conflict detection
- **Schema Validation**: JSON Schema validation for all API endpoints
- **Security**: HTML sanitization and XSS protection
- **Budget Enforcement**: Size limits for HTML (50KB) and JSON (500KB) files
- **CI/CD**: GitHub Actions workflow for automated builds and deployment

## Quick Start

### Using the Build Script

The easiest way to build the site:

```bash
./build.sh
```

Or with custom base URL:

```bash
BASE_URL="https://dl-alexandre.github.io/Bible/" ./build.sh
```

### Manual Build

```bash
cargo build --release
./target/release/bible-static-generator \
  --datasets datasets/kjv.txt \
  --datasets datasets/asv.txt \
  --datasets datasets/web.txt \
  --out out \
  --base-url "https://dl-alexandre.github.io/Bible/" \
  --minify-json \
  --gzip-json \
  --utc-timestamp \
  --threshold-jaccard 0.70 \
  --threshold-lev 0.15
```

### CLI Options

- `--datasets`: Path to dataset files (can be specified multiple times)
- `--out`: Output directory (default: `out`)
- `--base-url`: Base URL for sitemap (default: `https://example.com`)
- `--minify-json`: Minify JSON output
- `--gzip-json`: Compress JSON with gzip
- `--validate-only`: Validate schemas only (no generation)
- `--threshold-jaccard`: Jaccard similarity threshold (default: 0.70)
- `--threshold-lev`: Levenshtein distance threshold (default: 0.15)
- `--utc-timestamp`: Use UTC timestamp (normalized)
- `--schema-version`: Schema version (default: "1.0")
- `--templates`: Template directory
- `--log-dir`: Log directory (default: `logs`)

## Output Structure

```
out/
├── manifest.json          # Build manifest with metadata
├── versions.json          # Available versions
├── books.json             # Book metadata
├── crossrefs.json         # Cross-version mappings
├── sitemap.xml            # SEO sitemap
├── robots.txt              # Search engine directives
├── index.html              # Site index
├── schema/                 # JSON schemas
│   ├── manifest-1.0.json
│   ├── chapter-1.0.json
│   └── crossrefs-1.0.json
└── bible/                  # Version-specific content
    └── {version}/
        └── {book}/
            └── {chapter}.html
            └── {chapter}.json
```

## Data Provenance

### Source Licenses

Bible datasets used in this project are subject to their respective licenses:

- **KJV (King James Version)**: Public domain
- **WEB (World English Bible)**: Public domain
- **ASV (American Standard Version)**: Public domain

All source texts are verified for integrity using SHA-256 checksums, recorded in `manifest.json`.

### Build Provenance

Each build includes:
- Source dataset checksums (SHA-256)
- Build timestamp (UTC, normalized)
- Mapper thresholds (Jaccard, Levenshtein)
- Versification schemes
- Cross-reference map SHA-256 hash

## Development

### Running Tests

```bash
cargo test --release
```

### CI/CD

The project includes a GitHub Actions workflow (`.github/workflows/ci.yml`) that:
- Runs tests on Rust stable
- Validates schemas
- Checks output budgets
- Deploys to GitHub Pages

## License

This project is licensed under the MIT License. Source Bible texts retain their original public domain status.

