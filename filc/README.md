# Fil-C Core Prototype

This directory is the first migration boundary for issue #1. It is deliberately
small and does not replace the Rust generator.

Fil-C currently supports Linux on ARM64 and x86_64. Build this prototype on a
Linux Fil-C toolchain:

```sh
filc -std=c17 -Wall -Wextra -Werror -Iinclude -c src/bible_core.c -o bible_core.o
```

The first boundary owns normalized verse records only. Rust remains responsible
for parsing all existing source formats, mapping translations, schema checks,
and release generation until fixture parity is established.

## Parity milestone

The next step is a fixture test that feeds one chapter through both the Rust
parser and `bible_core_parse_line`, then compares book, chapter, verse, and text.
Do not connect this prototype to deployment until that fixture passes on Linux.
