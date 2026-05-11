#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECKPOINT_DIR="$ROOT_DIR/data/research_logs/checkpoints"
INTERVAL_SECONDS="${1:-1800}"
KEEP_COUNT="${2:-8}"

mkdir -p "$CHECKPOINT_DIR"

while true; do
  stamp="$(date +%Y%m%d-%H%M%S)"
  archive="$CHECKPOINT_DIR/corpus-auto-checkpoint-$stamp.tgz"
  tar -czf "$archive" \
    -C "$ROOT_DIR" \
    config/sources_manifest.json \
    data/chunks/constitution_full_corpus.json \
    data/clean \
    data/raw/bill_of_rights \
    data/raw/founders_correspondence \
    data/raw/letters_delegates_congress \
    data/delegates \
    data/bill_of_rights \
    scripts/fetch_founders_gap_sources.py \
    scripts/fetch_letters_delegates_loc.py \
    scripts/fill_founders_correspondence_gaps_from_published_sources.py

  ls -1t "$CHECKPOINT_DIR"/corpus-auto-checkpoint-*.tgz 2>/dev/null | tail -n +"$((KEEP_COUNT + 1))" | xargs -r rm -f
  sleep "$INTERVAL_SECONDS"
done
