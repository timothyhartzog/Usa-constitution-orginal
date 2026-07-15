import json
import urllib.request
import urllib.parse
from pathlib import Path

MANIFEST_PATH = Path("config/sources_manifest.json")
RAW_DIR = Path("data/raw/supreme_court_landmark_cases")

# A curated list of highly influential constitutional cases
LANDMARK_CASES = {
    "scotus_marbury_v_madison": ("5 U.S. 137", "Marbury v. Madison (1803)", "1803-02-24"),
    "scotus_mcculloch_v_maryland": ("17 U.S. 316", "McCulloch v. Maryland (1819)", "1819-03-06"),
    "scotus_gibbons_v_ogden": ("22 U.S. 1", "Gibbons v. Ogden (1824)", "1824-03-02"),
    "scotus_dred_scott_v_sandford": ("60 U.S. 393", "Dred Scott v. Sandford (1857)", "1857-03-06"),
    "scotus_plessy_v_ferguson": ("163 U.S. 537", "Plessy v. Ferguson (1896)", "1896-05-18"),
    "scotus_brown_v_board": ("347 U.S. 483", "Brown v. Board of Education (1954)", "1954-05-17"),
    "scotus_gideon_v_wainwright": ("372 U.S. 335", "Gideon v. Wainwright (1963)", "1963-03-18"),
    "scotus_miranda_v_arizona": ("384 U.S. 436", "Miranda v. Arizona (1966)", "1966-06-13"),
    "scotus_roe_v_wade": ("410 U.S. 113", "Roe v. Wade (1973)", "1973-01-22"),
    "scotus_us_v_nixon": ("418 U.S. 683", "United States v. Nixon (1974)", "1974-07-24"),
    "scotus_dc_v_heller": ("554 U.S. 570", "District of Columbia v. Heller (2008)", "2008-06-26"),
    "scotus_citizens_united_v_fec": ("558 U.S. 310", "Citizens United v. FEC (2010)", "2010-01-21"),
    "scotus_obergefell_v_hodges": ("576 U.S. 644", "Obergefell v. Hodges (2015)", "2015-06-26"),
    "scotus_dobbs_v_jackson": ("597 U.S. 215", "Dobbs v. Jackson Women's Health Organization (2022)", "2022-06-24"),
}

def fetch_opinion(citation):
    """Fetch the text of an opinion from Library of Congress using its citation."""
    try:
        # e.g. "5 U.S. 137" -> vol 5, reporter "U.S.", page 137
        parts = citation.split(" ")
        vol = parts[0]
        rep = parts[1].replace(".", "").lower() # U.S. -> us
        page = parts[2]
        
        # Format for LOC: https://www.loc.gov/item/usrep005137/
        loc_id = f"usrep{int(vol):03d}{int(page):03d}"
        url = f"https://www.loc.gov/item/{loc_id}/?fo=json"
        
        req = urllib.request.Request(url, headers={'User-Agent': 'ConstitutionalResearchSystem/1.0'})
        res = json.loads(urllib.request.urlopen(req).read())
        
        # LOC often provides a PDF or text array. Let's try to grab the text if available.
        # If not, we might need to fallback to Oyez or Justia via web scraping.
        # To avoid blocking, let's try scraping Justia directly as it's very reliable for SCOTUS text.
    except Exception as e:
        print(f"  Error fetching LOC {citation}: {e}")
        
    try:
        # Fallback to Justia
        parts = citation.split(" ")
        vol = parts[0]
        page = parts[2]
        url = f"https://supreme.justia.com/cases/federal/us/{vol}/{page}/"
        
        req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)'})
        html = urllib.request.urlopen(req).read().decode('utf-8')
        
        # Extremely basic extraction: grab text inside <div id="opinion-..."> or <div class="opinion">
        import re
        opinion_match = re.search(r'<div[^>]*id="opinion[^>]*>(.*?)</div>\s*<(div|footer)', html, re.DOTALL)
        if not opinion_match:
            opinion_match = re.search(r'<div[^>]*class="[^"]*opinion[^"]*"[^>]*>(.*?)</div>\s*<(div|footer)', html, re.DOTALL)
            
        if opinion_match:
            # Strip tags
            text = re.sub(r'<[^>]+>', ' ', opinion_match.group(1))
            text = re.sub(r'\s+', ' ', text).strip()
            return text
        else:
            print("  Could not find opinion div on Justia.")
            return None
    except Exception as e:
        print(f"  Error fetching Justia {citation}: {e}")
        return None

def main():
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    
    with open(MANIFEST_PATH, "r") as f:
        manifest = json.load(f)
    
    # Create or get collection
    collection = next((c for c in manifest["collections"] if c["collection_id"] == "supreme_court_landmark_cases"), None)
    if not collection:
        collection = {
            "collection_id": "supreme_court_landmark_cases",
            "title": "Landmark Supreme Court Cases",
            "documents": []
        }
        manifest["collections"].append(collection)
        print("Created supreme_court_landmark_cases collection in manifest.")

    existing_docs = {d["document_id"] for d in collection["documents"]}
    
    for doc_id, (citation, title, date) in LANDMARK_CASES.items():
        if doc_id not in existing_docs:
            collection["documents"].append({
                "document_id": doc_id,
                "title": title,
                "author": "Supreme Court of the United States",
                "date": date,
                "document_type": "case_law",
                "source_url": f"https://www.courtlistener.com/c/{citation.replace(' ', '')}",
                "source_format": "text",
                "chunk_strategy": "sliding_window",
                "default_issue_tags": ["case_law", "supreme_court"]
            })
            print(f"Added {doc_id} to manifest.")
            
        # Fetch if not already saved
        doc_dir = RAW_DIR / doc_id
        if not (doc_dir / "source.txt").exists():
            print(f"Fetching {title} ({citation})...")
            text = fetch_opinion(citation)
            if text:
                doc_dir.mkdir(parents=True, exist_ok=True)
                (doc_dir / "source.txt").write_text(text, encoding="utf-8")
                (doc_dir / "metadata.json").write_text(json.dumps({"status": "cached", "source_url": "courtlistener"}), encoding="utf-8")
                print(f"  Successfully scraped and saved {doc_id} ({len(text)} chars)")
            else:
                print(f"  Failed to fetch {title}.")
        else:
            print(f"Already have {doc_id} cached.")
            
    with open(MANIFEST_PATH, "w") as f:
        json.dump(manifest, f, indent=2)

if __name__ == '__main__':
    main()
