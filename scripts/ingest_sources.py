#!/usr/bin/env python3
"""
Ingest primary source documents from public archives.

Downloads documents from specified sources with fallback chain logic.
Stores raw texts with metadata about acquisition.
"""

import json
import os
import sys
from datetime import datetime
from pathlib import Path
import requests
from bs4 import BeautifulSoup
from urllib.parse import urljoin

# Project directories
PROJECT_ROOT = Path(__file__).parent.parent
DATA_RAW = PROJECT_ROOT / "data" / "raw"
CONFIG_DIR = PROJECT_ROOT / "config"
MANIFEST_PATH = CONFIG_DIR / "sources_manifest.json"

# Request configuration
REQUESTS_TIMEOUT = 30
HEADERS = {
    "User-Agent": "Constitutional Research System (Educational Use)"
}

class SourceAcquisition:
    """Manage source document acquisition with fallback logic."""

    def __init__(self):
        self.manifest = self._load_manifest()
        self.acquisition_report = {
            "timestamp": datetime.now().isoformat(),
            "total_documents": 0,
            "successful": 0,
            "failed": 0,
            "documents": []
        }

    def _load_manifest(self):
        """Load sources manifest configuration."""
        with open(MANIFEST_PATH, 'r') as f:
            return json.load(f)

    def _create_collection_dirs(self, collection_name, document_id):
        """Create directory structure for document."""
        doc_dir = DATA_RAW / collection_name / document_id
        doc_dir.mkdir(parents=True, exist_ok=True)
        return doc_dir

    def _download_url(self, url, timeout=REQUESTS_TIMEOUT):
        """Download content from URL with timeout and error handling."""
        try:
            response = requests.get(url, headers=HEADERS, timeout=timeout)
            response.raise_for_status()
            return response.text, response.status_code
        except requests.exceptions.Timeout:
            return None, "timeout"
        except requests.exceptions.ConnectionError:
            return None, "connection_error"
        except requests.exceptions.HTTPError as e:
            return None, f"http_{e.response.status_code}"
        except Exception as e:
            return None, f"error_{type(e).__name__}"

    def _extract_text_from_html(self, html_content):
        """Extract plain text from HTML using BeautifulSoup."""
        try:
            soup = BeautifulSoup(html_content, 'html.parser')
            # Remove script and style tags
            for tag in soup(['script', 'style']):
                tag.decompose()
            # Get text and clean up
            text = soup.get_text(separator='\n')
            # Clean up excessive whitespace
            lines = [line.strip() for line in text.split('\n')]
            lines = [line for line in lines if line]
            return '\n'.join(lines)
        except Exception as e:
            print(f"  Error extracting HTML: {e}")
            return None

    def _save_raw_text(self, doc_dir, content, format_type):
        """Save raw text to file."""
        if format_type == "text":
            filename = "source.txt"
        else:  # html
            filename = "source.html"

        filepath = doc_dir / filename
        try:
            with open(filepath, 'w', encoding='utf-8') as f:
                f.write(content)
            return str(filepath)
        except Exception as e:
            print(f"  Error saving file: {e}")
            return None

    def _acquire_document(self, collection, doc_spec):
        """Attempt to acquire a single document with fallback sources."""
        doc_id = doc_spec['id']
        title = doc_spec['title']
        sources = doc_spec.get('sources', [])

        print(f"\n  Acquiring: {title}")
        print(f"  Document ID: {doc_id}")

        # Create directory
        doc_dir = self._create_collection_dirs(collection, doc_id)

        # Try each source in priority order
        for source_idx, source in enumerate(sources, 1):
            url = source['url']
            format_type = source['format']
            priority = source['priority']

            print(f"    Trying source {source_idx} (priority {priority}): {url[:60]}...")

            # Download
            content, status = self._download_url(url)

            if content is None:
                print(f"      Failed: {status}")
                continue

            # Extract text if HTML
            if format_type == "html":
                text = self._extract_text_from_html(content)
                if text is None:
                    print(f"      Failed to extract text from HTML")
                    continue
            else:
                text = content

            # Save file
            filepath = self._save_raw_text(doc_dir, text, format_type)

            if filepath:
                char_count = len(text)
                word_count = len(text.split())

                print(f"      Success! ({word_count} words, {char_count} chars)")

                # Record in report
                self.acquisition_report['documents'].append({
                    "document_id": doc_id,
                    "title": title,
                    "collection": collection,
                    "status": "success",
                    "source_url": url,
                    "source_priority": priority,
                    "format": format_type,
                    "acquisition_timestamp": datetime.now().isoformat(),
                    "word_count": word_count,
                    "char_count": char_count,
                    "file_path": filepath
                })
                self.acquisition_report['successful'] += 1
                return True

        # All sources failed
        print(f"    Failed to acquire from all sources")
        self.acquisition_report['documents'].append({
            "document_id": doc_id,
            "title": title,
            "collection": collection,
            "status": "failed",
            "tried_sources": len(sources),
            "acquisition_timestamp": datetime.now().isoformat()
        })
        self.acquisition_report['failed'] += 1
        return False

    def acquire_all(self):
        """Acquire all documents from manifest."""
        print(f"\n{'='*70}")
        print("Constitutional Research System - Source Acquisition")
        print(f"{'='*70}")

        for source in self.manifest['sources']:
            collection = source['collection']
            documents = source['documents']

            print(f"\nCollection: {collection.replace('_', ' ').title()}")
            print(f"Documents: {len(documents)}")

            for doc in documents:
                self._acquire_document(collection, doc)
                self.acquisition_report['total_documents'] += 1

        # Save report
        self._save_report()

        # Print summary
        print(f"\n{'='*70}")
        print("ACQUISITION SUMMARY")
        print(f"{'='*70}")
        print(f"Total documents: {self.acquisition_report['total_documents']}")
        print(f"Successful: {self.acquisition_report['successful']}")
        print(f"Failed: {self.acquisition_report['failed']}")
        print(f"\nReport saved to: {PROJECT_ROOT / 'data' / 'acquisition_report.json'}")
        print(f"{'='*70}\n")

    def _save_report(self):
        """Save acquisition report to JSON."""
        report_path = PROJECT_ROOT / "data" / "acquisition_report.json"
        with open(report_path, 'w') as f:
            json.dump(self.acquisition_report, f, indent=2)

def main():
    """Main entry point."""
    # Ensure data directory exists
    DATA_RAW.mkdir(parents=True, exist_ok=True)

    # Run acquisition
    acquirer = SourceAcquisition()
    acquirer.acquire_all()

if __name__ == "__main__":
    main()
