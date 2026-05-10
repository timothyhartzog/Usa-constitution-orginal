#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.fetch_founders_online import load_metadata_records, write_document


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Import a browser-downloaded Founders Online JSONL batch.")
    parser.add_argument("batch_file", type=Path)
    parser.add_argument("--metadata-file", type=Path, required=True)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    records = {record.identifier: record for record in load_metadata_records(args.metadata_file, primary_only=True)}
    imported = 0
    with args.batch_file.open(encoding="utf-8") as handle:
        for line_number, line in enumerate(handle, start=1):
            if not line.strip():
                continue
            row = json.loads(line)
            identifier = row.get("identifier")
            data = row.get("data")
            if identifier not in records:
                raise SystemExit(f"line {line_number}: unknown identifier {identifier!r}")
            if not isinstance(data, dict):
                raise SystemExit(f"line {line_number}: data is not an object")
            write_document(records[identifier], data)
            imported += 1
    print(f"Imported {imported} documents from {args.batch_file}")


if __name__ == "__main__":
    main()
