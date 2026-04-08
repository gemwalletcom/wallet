#!/usr/bin/env bash
#
# Compute a content hash for Gemstone Rust sources.
# Uses git internals — near-instant vs find+cat.
#
# Usage:
#   gemstone-hash.sh <core-dir>           → prints hash to stdout
#   gemstone-hash.sh <core-dir> <file>    → writes hash to file

set -euo pipefail

CORE_DIR="$1"
OUT_FILE="${2:-}"

hash=$(
    {
        git -C "$CORE_DIR" rev-parse HEAD
        git -C "$CORE_DIR" diff -- crates/ gemstone/src/ Cargo.toml Cargo.lock gemstone/Cargo.toml
        git -C "$CORE_DIR" ls-files --others --exclude-standard -- crates/ gemstone/src/
    } | shasum -a 256 | cut -d' ' -f1
)

if [ -n "$OUT_FILE" ]; then
    mkdir -p "$(dirname "$OUT_FILE")"
    echo "$hash" > "$OUT_FILE"
else
    echo "$hash"
fi
