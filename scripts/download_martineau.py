import os
import urllib.request
import re
from pathlib import Path

OUT_DIR = Path("data/raw/martineau_illustrations")
OUT_DIR.mkdir(parents=True, exist_ok=True)

base_url = "https://oll.libertyfund.org"

for i in range(1, 10):
    vol_url = f"{base_url}/titles/martineau-illustrations-of-political-economy-vol-{i}"
    print(f"Fetching {vol_url}...")
    try:
        req = urllib.request.Request(vol_url, headers={'User-Agent': 'Mozilla/5.0'})
        html = urllib.request.urlopen(req).read().decode('utf-8')
        
        # Look for epub link
        match = re.search(r'href="(https://oll-resources\.s3[^"]+\.epub)"', html)
        if match:
            epub_url = match.group(1)
            out_file = OUT_DIR / f"vol_{i}.epub"
            print(f"  Downloading {epub_url} to {out_file}...")
            
            epub_req = urllib.request.Request(epub_url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(epub_req) as response, open(out_file, 'wb') as out:
                out.write(response.read())
            print(f"  Saved Volume {i}.")
        else:
            print(f"  No epub link found for volume {i}.")
            
            # look for HTML reading link?
            match_html = re.search(r'href="([^"]+)"[^>]*>HTML', html)
            if match_html:
                print("  Found HTML link instead:", match_html.group(1))

    except Exception as e:
        print(f"  Error fetching volume {i}: {e}")

print("Download complete.")
