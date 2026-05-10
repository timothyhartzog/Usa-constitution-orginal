#!/usr/bin/env python3
from __future__ import annotations

import argparse
import html
import json
import sys
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.fetch_founders_online import (
    CLEAN_DIR,
    RAW_DIR,
    MetadataRecord,
    load_metadata_records,
    local_doc_path,
    write_document,
)
from scripts.utils.pipeline import ensure_directory


class SinkState:
    def __init__(self, records: list[MetadataRecord], offset: int) -> None:
        self.records = records
        self.cursor = offset
        self.saved = 0
        self.skipped = 0

    def next_record(self) -> MetadataRecord | None:
        while self.cursor < len(self.records):
            record = self.records[self.cursor]
            self.cursor += 1
            raw_path = local_doc_path(RAW_DIR, record.identifier, "json")
            clean_path = local_doc_path(CLEAN_DIR, record.identifier, "txt")
            if raw_path.exists() and clean_path.exists():
                self.skipped += 1
                continue
            return record
        return None


def json_response(handler: BaseHTTPRequestHandler, status: int, payload: object) -> None:
    body = json.dumps(payload, ensure_ascii=False).encode("utf-8")
    handler.send_response(status)
    handler.send_header("Content-Type", "application/json; charset=utf-8")
    handler.send_header("Content-Length", str(len(body)))
    handler.send_header("Access-Control-Allow-Origin", "https://founders.archives.gov")
    handler.send_header("Access-Control-Allow-Methods", "GET,POST,OPTIONS")
    handler.send_header("Access-Control-Allow-Headers", "Content-Type")
    handler.send_header("Access-Control-Allow-Private-Network", "true")
    handler.end_headers()
    handler.wfile.write(body)


def make_handler(state: SinkState):
    class FoundersSinkHandler(BaseHTTPRequestHandler):
        def do_OPTIONS(self) -> None:
            json_response(self, 204, {})

        def do_GET(self) -> None:
            parsed = urlparse(self.path)
            if parsed.path == "/next":
                record = state.next_record()
                if not record:
                    json_response(self, 200, {"done": True, "cursor": state.cursor, "saved": state.saved, "skipped": state.skipped})
                    return
                json_response(
                    self,
                    200,
                    {
                        "done": False,
                        "cursor": state.cursor,
                        "identifier": record.identifier,
                        "title": record.title,
                        "permalink": record.permalink,
                    },
                )
                return

            if parsed.path == "/status":
                raw_count = sum(1 for _ in RAW_DIR.rglob("*.json")) if RAW_DIR.exists() else 0
                json_response(
                    self,
                    200,
                    {
                        "cursor": state.cursor,
                        "saved_this_session": state.saved,
                        "skipped_this_session": state.skipped,
                        "raw_files": raw_count,
                        "total_records": len(state.records),
                    },
                )
                return

            if parsed.path == "/script":
                query = parse_qs(parsed.query)
                delay_ms = int(query.get("delay_ms", ["2000"])[0])
                script = browser_script(delay_ms)
                body = script.encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/javascript; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(body)
                return

            if parsed.path == "/launcher":
                body = launcher_page().encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "text/html; charset=utf-8")
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
                return

            json_response(self, 404, {"error": "not found"})

        def do_POST(self) -> None:
            if urlparse(self.path).path != "/save":
                json_response(self, 404, {"error": "not found"})
                return
            length = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            identifier = payload.get("identifier")
            data = payload.get("data")
            if not isinstance(identifier, str) or not isinstance(data, dict):
                json_response(self, 400, {"error": "expected identifier and data object"})
                return
            record = next((candidate for candidate in state.records if candidate.identifier == identifier), None)
            if not record:
                json_response(self, 404, {"error": f"unknown identifier {identifier}"})
                return
            raw_path, clean_path = write_document(record, data)
            state.saved += 1
            json_response(
                self,
                200,
                {
                    "saved": True,
                    "identifier": identifier,
                    "raw_path": str(raw_path.relative_to(PROJECT_ROOT)),
                    "clean_path": str(clean_path.relative_to(PROJECT_ROOT)),
                    "saved_this_session": state.saved,
                },
            )

        def log_message(self, format: str, *args: object) -> None:
            return

    return FoundersSinkHandler


def browser_script(delay_ms: int) -> str:
    return f"""
(async () => {{
  const sink = "http://127.0.0.1:8765";
  const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
  let saved = 0;
  let failed = 0;
  console.log("Founders browser downloader started");
  if (location.hostname === "127.0.0.1" || location.hostname === "localhost") {{
    const message = `Wrong page for downloader. Current page is ${{location.href}}. Open a founders.archives.gov API page, then click the bookmark there.`;
    console.error(message, location.href);
    alert(message);
    return;
  }}
  while (true) {{
    console.log("asking local sink for next document");
    const nextResponse = await fetch(`${{sink}}/next`);
    console.log("local /next status", nextResponse.status);
    const next = await nextResponse.json();
    console.log("local /next payload", next);
    if (next.done) {{
      console.log("Founders browser downloader finished", next);
      break;
    }}
    const apiUrl = `https://founders.archives.gov/API/docdata/${{next.identifier}}`;
    console.log("fetching Founders API", apiUrl);
    try {{
      const response = await fetch(apiUrl, {{ credentials: "include" }});
      console.log("Founders API status", response.status, response.headers.get("content-type"));
      if (!response.ok) {{
        failed += 1;
        console.error("API failed", response.status, next.identifier);
        break;
      }}
      const data = await response.json();
      console.log("saving locally", next.identifier);
      const saveResponse = await fetch(`${{sink}}/save`, {{
        method: "POST",
        headers: {{ "Content-Type": "application/json" }},
        body: JSON.stringify({{ identifier: next.identifier, data }}),
      }});
      console.log("local /save status", saveResponse.status);
      const savePayload = await saveResponse.json();
      console.log("local /save payload", savePayload);
      if (!saveResponse.ok) {{
        failed += 1;
        console.error("save failed", savePayload);
        break;
      }}
      saved += 1;
      if (saved === 1 || saved % 50 === 0) {{
        console.log(`saved ${{saved}} docs; latest ${{next.identifier}}`);
      }}
      await delay({delay_ms});
    }} catch (error) {{
      failed += 1;
      console.error("Downloader stopped", next.identifier, error);
      break;
    }}
  }}
  console.log("Founders browser downloader summary", {{ saved, failed }});
}})();
""".strip()


def launcher_page() -> str:
    bookmarklet = html.escape("javascript:" + browser_script(2000).replace("\n", " "), quote=True)
    return f"""
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>Founders Online Downloader Launcher</title>
  <style>
    body {{ font: 16px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; max-width: 760px; margin: 48px auto; padding: 0 24px; }}
    a.bookmarklet {{ display: inline-block; padding: 10px 14px; border: 1px solid #333; border-radius: 6px; color: #111; text-decoration: none; }}
    code {{ background: #f4f4f4; padding: 2px 4px; border-radius: 4px; }}
    li {{ margin: 10px 0; }}
  </style>
</head>
<body>
  <h1>Founders Online Downloader Launcher</h1>
  <ol>
    <li>Show Chrome's bookmarks bar with <code>Shift + Command + B</code>.</li>
    <li>Drag this link to the bookmarks bar: <a class="bookmarklet" href="{bookmarklet}">Start Founders Download</a></li>
    <li>Open a Founders Online API page, for example <code>https://founders.archives.gov/API/docdata/Adams/01-01-02-0002-0006-0006</code>.</li>
    <li>Click the <strong>Start Founders Download</strong> bookmark.</li>
    <li>Leave the tab open. Progress is saved to this local server.</li>
  </ol>
</body>
</html>
""".strip()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Local sink for a Chrome-console Founders Online downloader.")
    parser.add_argument("--metadata-file", type=Path, required=True)
    parser.add_argument("--offset", type=int, default=0)
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    ensure_directory(RAW_DIR)
    ensure_directory(CLEAN_DIR)
    records = load_metadata_records(args.metadata_file, primary_only=True)
    state = SinkState(records, args.offset)
    server = ThreadingHTTPServer((args.host, args.port), make_handler(state))
    print(f"Loaded {len(records)} records")
    print(f"Listening on http://{args.host}:{args.port}")
    print(f"Browser script: http://{args.host}:{args.port}/script?delay_ms=2000")
    server.serve_forever()


if __name__ == "__main__":
    main()
