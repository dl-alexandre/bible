#!/bin/sh

set -eu

BASE_URL="${BASE_URL:-https://dl-alexandre.ddns.net/bible/}"
OUT_DIR="${OUT_DIR:-out}"
LOCAL_ROOT="${LOCAL_ROOT:-/Users/developer/Public/bible}"

BASE_URL="$BASE_URL" OUT_DIR="$OUT_DIR" ./scripts/build.sh
rm -rf "$LOCAL_ROOT"
mkdir -p "$LOCAL_ROOT"
cp -R "$OUT_DIR/bible/." "$LOCAL_ROOT/"
printf 'Local Bible deployed to %s\n' "$LOCAL_ROOT"
