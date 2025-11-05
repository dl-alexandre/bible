# Deployment Guide

## Quick Setup

1. **Create GitHub Repository**
   ```bash
   # Create a new repository on GitHub named "Bible"
   # Then connect it:
   git remote add origin https://github.com/dl-alexandre/Bible.git
   git push -u origin master
   ```

2. **Enable GitHub Pages**
   - Go to repository Settings → Pages
   - Source: GitHub Actions
   - The workflow will automatically deploy on push

## Building Locally

### Quick Build
```bash
./scripts/build.sh
```

### Custom Base URL
```bash
BASE_URL="https://your-custom-url.com" ./scripts/build.sh
```

### Manual Build
```bash
cargo build --release
./target/release/bible-static-generator \
  --datasets datasets/kjv.txt \
  --datasets datasets/asv.txt \
  --datasets datasets/web.txt \
  --out out \
  --base-url "https://dl-alexandre.github.io/bible/" \
  --minify-json \
  --gzip-json \
  --utc-timestamp \
  --threshold-jaccard 0.70 \
  --threshold-lev 0.15
```

## CLI Reference

### Required Options
- `--datasets`: Bible dataset files (specify multiple times)
- `--out`: Output directory

### Recommended Options
- `--base-url`: Your GitHub Pages URL (e.g., `https://dl-alexandre.github.io/bible/`)
- `--minify-json`: Minify JSON output files
- `--gzip-json`: Compress JSON files
- `--utc-timestamp`: Use normalized UTC timestamps

### Advanced Options
- `--threshold-jaccard`: Similarity threshold (default: 0.70)
- `--threshold-lev`: Distance threshold (default: 0.15)
- `--schema-version`: Schema version (default: "1.0")
- `--templates`: Custom template directory
- `--log-dir`: Log directory (default: `logs`)
- `--validate-only`: Validate schemas without generating

## Output

The build generates:
- **HTML files**: `out/bible/{version}/{book}/{chapter}.html`
- **Index page**: `out/index.html`
- **Sitemap**: `out/sitemap.xml`
- **Manifest**: `out/manifest.json`
- **Static assets**: `out/static/`

JSON chapter files are automatically removed (HTML-only mode).

## GitHub Actions

The workflow automatically:
1. Builds the Rust binary
2. Generates the site with correct base URL
3. Validates output
4. Deploys to GitHub Pages

No manual steps needed after pushing!

