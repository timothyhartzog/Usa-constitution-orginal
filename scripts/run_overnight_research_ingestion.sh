#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

mkdir -p data/research_logs
LOG="data/research_logs/overnight_research_ingestion.log"

{
  echo "started_at=$(date -Iseconds)"
  echo "repo=$ROOT"

  echo "step=delegate_coverage"
  python3 scripts/delegate_coverage_report.py --top 25

  echo "step=delegate_dossiers"
  python3 scripts/build_delegate_dossiers.py --top 100

  echo "step=us_to_world_comparison"
  python3 scripts/compare_us_to_world_constitutions.py --top 50

  echo "step=graph_and_vectors"
  python3 -u scripts/build_research_artifacts.py --max-features 50000 --max-terms-per-chunk 192

  echo "step=tests"
  if [[ -x .venv/bin/python ]]; then
    .venv/bin/python -m pytest tests/test_manifest_schema.py tests/test_uniqueness.py tests/test_urls.py
  else
    python3 -m pytest tests/test_manifest_schema.py tests/test_uniqueness.py tests/test_urls.py
  fi

  echo "finished_at=$(date -Iseconds)"
} > "$LOG" 2>&1

echo "Saved $LOG"
