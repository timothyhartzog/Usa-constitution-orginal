#!/usr/bin/env python3
"""
Semantic Vector Search using Legal-BERT and FAISS.
Queries the vector index for semantically related passages.
"""

import json
import argparse
from pathlib import Path
import warnings
import faiss
import numpy as np

# Suppress HuggingFace warnings
warnings.filterwarnings("ignore")

try:
    import torch
    from sentence_transformers import SentenceTransformer
except ImportError:
    print("Required ML libraries are missing. Run: pip install -r requirements.txt")
    exit(1)

PROJECT_ROOT = Path(__file__).parent.parent
CORPUS_PATH = PROJECT_ROOT / "data" / "chunks" / "constitution_full_corpus.json"
INDEX_DIR = PROJECT_ROOT / "data" / "index"
FAISS_INDEX_PATH = INDEX_DIR / "vector_index.faiss"
MAPPING_PATH = INDEX_DIR / "vector_mapping.json"
MODEL_NAME = "nlpaueb/legal-bert-base-uncased"

def get_device():
    if torch.backends.mps.is_available():
        return "mps"
    elif torch.cuda.is_available():
        return "cuda"
    return "cpu"

def load_data():
    if not FAISS_INDEX_PATH.exists() or not MAPPING_PATH.exists():
        raise SystemExit("Vector index not found. Run scripts/build_vector_index.py first.")
        
    index = faiss.read_index(str(FAISS_INDEX_PATH))
    with open(MAPPING_PATH) as f:
        mapping = json.load(f)["chunk_ids"]
        
    with open(CORPUS_PATH) as f:
        corpus = json.load(f).get("chunks", [])
        
    chunk_map = {c["chunk_id"]: c for c in corpus}
    return index, mapping, chunk_map

def search(query, limit=5):
    index, mapping, chunk_map = load_data()
    device = get_device()
    
    # Load model silently
    model = SentenceTransformer(MODEL_NAME, device=device)
    
    # Encode query
    query_vector = model.encode([query], convert_to_numpy=True)
    query_vector = np.array(query_vector).astype("float32")
    faiss.normalize_L2(query_vector)
    
    # Search FAISS
    scores, indices = index.search(query_vector, limit)
    
    print(f"\n🔍 SEMANTIC SEARCH RESULTS FOR: '{query}'\n" + "="*60)
    for i, (score, idx) in enumerate(zip(scores[0], indices[0])):
        if idx == -1: continue
        chunk_id = mapping[idx]
        chunk = chunk_map.get(chunk_id, {})
        text = chunk.get("text", "")[:300].replace('\n', ' ') + "..."
        source = chunk.get("source_collection", "unknown")
        
        print(f"[{i+1}] Score: {score:.4f} | {chunk_id} ({source})")
        print(f"    {text}\n")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Query the Semantic Vector Index")
    parser.add_argument("query", type=str, help="The search query")
    parser.add_argument("--limit", type=int, default=5, help="Number of results to return")
    args = parser.parse_args()
    
    search(args.query, limit=args.limit)
