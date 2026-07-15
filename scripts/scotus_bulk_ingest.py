import os
import json
import urllib.request
import urllib.parse
from pathlib import Path
import zipfile
import io
import csv

# Destination directory for the bulk data
RAW_DIR = Path("data/raw/scotus_bulk")
API_URL = "http://127.0.0.1:8082/api/documents/bulk"
CRS_API_KEY = os.environ.get("CRS_API_KEY", "secret-token")
COURTLISTENER_API_KEY = os.environ.get("COURTLISTENER_API_KEY", "")

class ScotusIngestor:
    """Methods to pull bulk Supreme Court documents from various open source repositories."""
    
    def __init__(self):
        RAW_DIR.mkdir(parents=True, exist_ok=True)
        self.bulk_batch = []
        self.batch_size = 25

    def push_batch(self, force=False):
        if not self.bulk_batch:
            return
            
        if len(self.bulk_batch) >= self.batch_size or force:
            req_data = json.dumps({"documents": self.bulk_batch}).encode('utf-8')
            req = urllib.request.Request(API_URL, data=req_data, headers={
                'Content-Type': 'application/json',
                'Authorization': f'Bearer {CRS_API_KEY}'
            }, method='POST')
            
            try:
                response = urllib.request.urlopen(req)
                print(f"  [API] Successfully pushed batch of {len(self.bulk_batch)} documents.")
            except Exception as e:
                print(f"  [API] Failed to push batch: {e}")
                
            self.bulk_batch = []

    def queue_document(self, doc):
        self.bulk_batch.append(doc)
        self.push_batch()

    def pull_rosthalken_csv(self):
        print("Pulling from rosthalken/supreme-court-data...")
        sample_csv_path = RAW_DIR / "rosthalken_opinions.csv"
        
        if not sample_csv_path.exists():
            print("  [Skipping] CSV not found locally. Generate using the rosthalken python pipeline first.")
            return

        with open(sample_csv_path, "r", encoding="utf-8") as f:
            reader = csv.DictReader(f)
            count = 0
            for row in reader:
                self.queue_document({
                    "title": row.get("title", "Unknown Case"),
                    "author": row.get("author", "Supreme Court"),
                    "date": row.get("date", ""),
                    "source_collection": "rosthalken_scotus_data",
                    "document_type": "case_law",
                    "text": row.get("text", "")
                })
                count += 1
            self.push_batch(force=True)
            print(f"  Processed and pushed {count} cases from CSV.")


    def pull_ericwiener_transcripts(self):
        print("Pulling transcripts from EricWiener/supreme-court-cases...")
        
        repo_url = "https://github.com/EricWiener/supreme-court-cases/archive/refs/heads/master.zip"
        target_dir = RAW_DIR / "ericwiener_transcripts"
        
        if not target_dir.exists():
            target_dir.mkdir(parents=True)
            try:
                print(f"  Downloading repo archive from {repo_url}...")
                req = urllib.request.Request(repo_url, headers={'User-Agent': 'Mozilla/5.0'})
                with urllib.request.urlopen(req) as response:
                    with zipfile.ZipFile(io.BytesIO(response.read())) as z:
                        for file_info in z.infolist():
                            # The transcript files are saved as .js in the cases/ directory
                            if (file_info.filename.endswith('.js') or file_info.filename.endswith('.json')) and '/cases/' in file_info.filename:
                                z.extract(file_info, target_dir)
                print("  Extraction complete.")
            except Exception as e:
                print(f"  Error downloading repository: {e}")
                return
        
        # Process extracted files (.js or .json)
        json_files = list(target_dir.rglob("*.js")) + list(target_dir.rglob("*.json"))
        print(f"  Found {len(json_files)} transcript JSON/JS files.")
        
        for jpath in json_files:
            with open(jpath, "r", encoding="utf-8") as f:
                try:
                    data = json.load(f)
                    title = data.get("caseName", jpath.stem)
                    date = data.get("term", "")
                    
                    transcript_text = ""
                    for transcript in data.get("caseTranscripts", []):
                        for speech in transcript.get("transcript", []):
                            speaker = speech.get("speakerName", "Unknown")
                            text_objs = speech.get("textObjs", [])
                            # Usually there's an array of text objects with cleanText
                            for to in text_objs:
                                text = to.get("text", "")
                                if text:
                                    transcript_text += f"[{speaker}]: {text}\n\n"
                                    
                    print(f"  Parsed transcript for: {title} ({len(transcript_text)} chars)")
                    
                    self.queue_document({
                        "title": f"{title} (Oral Argument Transcript)",
                        "author": "Supreme Court / Advocates",
                        "date": date,
                        "source_collection": "ericwiener_transcripts",
                        "document_type": "transcript",
                        "text": transcript_text
                    })
                except json.JSONDecodeError:
                    pass
        
        self.push_batch(force=True)


    def pull_freelawproject_courtlistener(self, max_items=10):
        print("Pulling from Free Law Project (CourtListener API)...")
        if not COURTLISTENER_API_KEY:
            print("  [Skipping] COURTLISTENER_API_KEY environment variable is not set. Required for opinions API.")
            return

        url = "https://www.courtlistener.com/api/rest/v3/opinions/?court=scotus"
        
        try:
            req = urllib.request.Request(url, headers={
                'User-Agent': 'ConstitutionalResearchSystem/1.0',
                'Authorization': f'Token {COURTLISTENER_API_KEY}'
            })
            response = urllib.request.urlopen(req)
            data = json.loads(response.read())
            
            results = data.get("results", [])
            print(f"  Retrieved page 1 with {len(results)} opinions.")
            
            for item in results[:max_items]:
                text = item.get("plain_text") or item.get("html_with_citations") or item.get("html_lawbox") or ""
                doc_id = item.get("id")
                
                print(f"  Processing CourtListener Opinion ID: {doc_id} (Length: {len(text)})")
                
                self.queue_document({
                    "title": f"CourtListener Opinion {doc_id}",
                    "author": "Supreme Court",
                    "date": "",
                    "source_collection": "courtlistener",
                    "document_type": "case_law",
                    "text": text
                })
                
            self.push_batch(force=True)
                    
        except Exception as e:
            print(f"  Error fetching from CourtListener: {e}")


def main():
    ingestor = ScotusIngestor()
    
    # 1. Pull CSVs from rosthalken repo
    ingestor.pull_rosthalken_csv()
    print("-" * 40)
    
    # 2. Pull Transcripts from EricWiener
    ingestor.pull_ericwiener_transcripts()
    print("-" * 40)
    
    # 3. Pull from CourtListener / Free Law Project
    ingestor.pull_freelawproject_courtlistener()

if __name__ == "__main__":
    main()
