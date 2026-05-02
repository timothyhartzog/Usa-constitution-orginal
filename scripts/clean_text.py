#!/usr/bin/env python3
"""
Clean and normalize raw text from primary sources.

Removes OCR artifacts, standardizes whitespace, fixes encoding issues.
Preserves original language and structure.
"""

import json
import re
from pathlib import Path
from datetime import datetime

# Project directories
PROJECT_ROOT = Path(__file__).parent.parent
DATA_RAW = PROJECT_ROOT / "data" / "raw"
DATA_CLEAN = PROJECT_ROOT / "data" / "clean"
MANIFEST_PATH = PROJECT_ROOT / "config" / "sources_manifest.json"

class TextCleaner:
    """Clean and normalize historical texts."""

    def __init__(self):
        self.manifest = self._load_manifest()
        self.clean_report = {
            "timestamp": datetime.now().isoformat(),
            "total_documents": 0,
            "successful": 0,
            "failed": 0,
            "documents": []
        }

    def _load_manifest(self):
        """Load sources manifest."""
        with open(MANIFEST_PATH, 'r') as f:
            return json.load(f)

    def _read_raw_file(self, filepath):
        """Read raw text file with encoding detection."""
        try:
            # Try UTF-8 first
            with open(filepath, 'r', encoding='utf-8') as f:
                return f.read()
        except UnicodeDecodeError:
            # Fall back to latin-1
            try:
                with open(filepath, 'r', encoding='latin-1') as f:
                    return f.read()
            except Exception as e:
                print(f"    Error reading file: {e}")
                return None

    def _clean_text(self, text):
        """Apply cleaning transformations."""
        transformations = []
        original_length = len(text)

        # Remove excessive blank lines (keep max 2 consecutive)
        pattern = r'\n\n\n+'
        if re.search(pattern, text):
            text = re.sub(pattern, '\n\n', text)
            transformations.append("removed_excessive_blank_lines")

        # Remove common OCR artifacts
        if re.search(r'[\-_]{4,}', text):  # Multiple dashes/underscores
            text = re.sub(r'[\-_]{4,}', '', text)
            transformations.append("removed_ocr_dash_artifacts")

        if re.search(r'={4,}', text):  # Multiple equals signs
            text = re.sub(r'={4,}', '', text)
            transformations.append("removed_ocr_equals_artifacts")

        # Remove page numbers and page markers (common OCR patterns)
        # Pattern: standalone numbers, often with dashes
        text = re.sub(r'\n\s*\d{1,4}\s*\n', '\n', text)
        transformations.append("removed_page_numbers")

        # Remove headers/footers that are just repeated text fragments
        # (This is conservative - only removes obvious duplicates)

        # Normalize whitespace while preserving structure
        # Replace multiple spaces with single space
        lines = text.split('\n')
        normalized_lines = []

        for line in lines:
            # Remove trailing whitespace
            line = line.rstrip()
            # Replace multiple spaces with single space
            line = re.sub(r' {2,}', ' ', line)
            normalized_lines.append(line)

        text = '\n'.join(normalized_lines)
        transformations.append("normalized_whitespace")

        # Remove leading/trailing whitespace from entire document
        text = text.strip()

        # Fix common encoding issues
        replacements = {
            '\x00': '',  # Null bytes
            '﻿': '',  # Byte order mark
        }
        for bad_char, replacement in replacements.items():
            if bad_char in text:
                text = text.replace(bad_char, replacement)
                transformations.append(f"fixed_encoding_char_{ord(bad_char)}")

        # Normalize common quote characters
        text = text.replace('‘', "'")  # Left single quote
        text = text.replace('’', "'")  # Right single quote
        text = text.replace('“', '"')  # Left double quote
        text = text.replace('”', '"')  # Right double quote
        transformations.append("normalized_quotes")

        # Normalize line endings to \n
        text = text.replace('\r\n', '\n').replace('\r', '\n')

        final_length = len(text)
        char_diff = original_length - final_length

        return text, transformations, char_diff

    def _clean_document(self, collection, doc_spec):
        """Clean a single document."""
        doc_id = doc_spec['id']
        title = doc_spec['title']

        print(f"\n  Cleaning: {title}")

        # Find raw file
        raw_dir = DATA_RAW / collection / doc_id
        raw_file = None

        for fname in ['source.txt', 'source.html']:
            candidate = raw_dir / fname
            if candidate.exists():
                raw_file = candidate
                break

        if raw_file is None:
            print(f"    No raw file found")
            self.clean_report['documents'].append({
                "document_id": doc_id,
                "title": title,
                "collection": collection,
                "status": "failed",
                "reason": "no_raw_file"
            })
            return False

        # Read raw file
        text = self._read_raw_file(raw_file)
        if text is None:
            print(f"    Failed to read raw file")
            self.clean_report['documents'].append({
                "document_id": doc_id,
                "title": title,
                "collection": collection,
                "status": "failed",
                "reason": "read_error"
            })
            return False

        original_word_count = len(text.split())

        # Clean text
        cleaned_text, transformations, char_diff = self._clean_text(text)

        cleaned_word_count = len(cleaned_text.split())
        word_diff = original_word_count - cleaned_word_count

        print(f"    Original: {original_word_count} words")
        print(f"    Cleaned: {cleaned_word_count} words (diff: {word_diff})")
        print(f"    Transformations: {len(transformations)}")

        # Save cleaned text
        DATA_CLEAN.mkdir(parents=True, exist_ok=True)
        clean_file = DATA_CLEAN / f"{doc_id}.txt"

        try:
            with open(clean_file, 'w', encoding='utf-8') as f:
                f.write(cleaned_text)
        except Exception as e:
            print(f"    Error writing clean file: {e}")
            self.clean_report['documents'].append({
                "document_id": doc_id,
                "title": title,
                "collection": collection,
                "status": "failed",
                "reason": f"write_error_{e}"
            })
            return False

        # Record success
        self.clean_report['documents'].append({
            "document_id": doc_id,
            "title": title,
            "collection": collection,
            "status": "success",
            "original_word_count": original_word_count,
            "cleaned_word_count": cleaned_word_count,
            "word_diff": word_diff,
            "char_diff": char_diff,
            "transformations": transformations,
            "transformations_count": len(transformations),
            "output_file": str(clean_file)
        })
        self.clean_report['successful'] += 1
        return True

    def clean_all(self):
        """Clean all documents from manifest."""
        print(f"\n{'='*70}")
        print("Constitutional Research System - Text Cleaning")
        print(f"{'='*70}")

        for source in self.manifest['sources']:
            collection = source['collection']
            documents = source['documents']

            print(f"\nCollection: {collection.replace('_', ' ').title()}")

            for doc in documents:
                self._clean_document(collection, doc)
                self.clean_report['total_documents'] += 1

        # Save report
        self._save_report()

        # Print summary
        print(f"\n{'='*70}")
        print("CLEANING SUMMARY")
        print(f"{'='*70}")
        print(f"Total documents: {self.clean_report['total_documents']}")
        print(f"Successful: {self.clean_report['successful']}")
        print(f"Failed: {self.clean_report['failed']}")
        print(f"\nReport saved to: {PROJECT_ROOT / 'data' / 'clean_report.json'}")
        print(f"{'='*70}\n")

    def _save_report(self):
        """Save cleaning report."""
        report_path = PROJECT_ROOT / "data" / "clean_report.json"
        with open(report_path, 'w') as f:
            json.dump(self.clean_report, f, indent=2)
            self.clean_report['failed'] += 1

def main():
    """Main entry point."""
    cleaner = TextCleaner()
    cleaner.clean_all()

if __name__ == "__main__":
    main()
