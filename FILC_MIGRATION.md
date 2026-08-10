# Fil-C Migration

## Current Status

The generator remains the Rust implementation. Fil-C currently supports Linux
on ARM64 or x86_64, while the deployment host is macOS ARM64. There is no Fil-C
compiler or Linux container runtime installed on this host, so a native rewrite
cannot be compiled or verified here.

The existing static site remains the release path while the migration is
developed separately.

## Migration Boundary

The rewrite should preserve these externally visible contracts:

- deterministic HTML and JSON output
- KJV, ASV, WEB, and BSB dataset parsing
- cross-version mapping and configured similarity thresholds
- schema validation and output budget checks
- the existing `out/bible/` Pages layout

The first Fil-C milestone should be a command-line parser and validator that
reads one dataset and emits the same normalized intermediate representation as
the Rust parser. Output fixtures from the Rust generator should be used as the
compatibility oracle before moving mapping or HTML generation.

## Work Plan

1. Provision a Linux ARM64 or x86_64 builder with Fil-C from
   `https://github.com/pizlonator/fil-c`.
2. Add a small Fil-C core with a stable C ABI for dataset parsing and normalized
   verse records.
3. Build fixture-based equivalence tests against the Rust parser.
4. Port mapping, validation, and generation one subsystem at a time.
5. Keep Rust as the fallback until the complete output fixture suite passes.
6. Add Fil-C builds to CI before switching GitHub Pages deployment.

## Non-Goals For The First Port

- replacing the working deployment before output parity
- adding WASM before the native Fil-C core is validated
- changing the public URL or generated directory layout
