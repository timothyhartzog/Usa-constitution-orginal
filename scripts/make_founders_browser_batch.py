#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.fetch_founders_online import RAW_DIR, CLEAN_DIR, load_metadata_records, local_doc_path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate pasteable browser JavaScript for a Founders Online batch download.")
    parser.add_argument("--metadata-file", type=Path, required=True)
    parser.add_argument("--offset", type=int, default=0)
    parser.add_argument("--limit", type=int, default=25)
    parser.add_argument("--delay-ms", type=int, default=2000)
    parser.add_argument("--out", type=Path, default=PROJECT_ROOT / "data" / "founders_online" / "browser_batch.js")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    records = load_metadata_records(args.metadata_file, primary_only=True)
    selected = []
    cursor = args.offset
    while cursor < len(records) and len(selected) < args.limit:
        record = records[cursor]
        cursor += 1
        raw_path = local_doc_path(RAW_DIR, record.identifier, "json")
        clean_path = local_doc_path(CLEAN_DIR, record.identifier, "txt")
        if raw_path.exists() and clean_path.exists():
            continue
        selected.append({"identifier": record.identifier, "title": record.title})

    payload = json.dumps(selected, ensure_ascii=False)
    filename = f"founders_batch_offset_{args.offset}_limit_{len(selected)}.jsonl"
    script = f"""
(async function() {{
  const docs = {payload};
  const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  const rows = [];
  console.log("Founders batch started", {{ count: docs.length }});
  for (let i = 0; i < docs.length; i += 1) {{
    const doc = docs[i];
    const url = "https://founders.archives.gov/API/docdata/" + doc.identifier;
    console.log(String(i + 1) + "/" + String(docs.length), doc.identifier);
    const response = await fetch(url, {{ credentials: "include" }});
    if (!response.ok) {{
      console.error("Stopped on API status", response.status, doc.identifier);
      break;
    }}
    const data = await response.json();
    rows.push(JSON.stringify({{ identifier: doc.identifier, data }}));
    await delay({args.delay_ms});
  }}
  const blob = new Blob([rows.join("\\n") + "\\n"], {{ type: "application/x-ndjson" }});
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = "{filename}";
  document.body.appendChild(link);
  link.click();
  URL.revokeObjectURL(link.href);
  link.remove();
  console.log("Founders batch downloaded", {{ rows: rows.length, filename: "{filename}" }});
}})();
""".strip()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(script, encoding="utf-8")
    print(f"Wrote {args.out}")
    print(f"Selected {len(selected)} missing records from offset {args.offset}; next cursor {cursor}")
    print(f"Expected download filename: {filename}")


if __name__ == "__main__":
    main()
