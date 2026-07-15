#!/usr/bin/env python3
"""
Build Vector Index using Legal-BERT
Extracts text chunks, encodes them into high-dimensional semantic vectors using
the Legal-BERT model, and saves them to a FAISS index for semantic search.
"""

import json
import time
import argparse
from pathlib import Path

try:
    import torch
    import faiss
    import numpy as np
    from tqdm import tqdm
    from sentence_transformers import SentenceTransformer
except ImportError:
    print("Required ML libraries are missing. Run: pip install -r requirements.txt")
    exit(1)

PROJECT_ROOT = Path(__file__).parent.parent
CORPUS_PATH = PROJECT_ROOT / "data" / "chunks" / "constitution_full_corpus.json"
INDEX_DIR = PROJECT_ROOT / "data" / "index"
FAISS_INDEX_PATH = INDEX_DIR / "vector_index.faiss"
MAPPING_PATH = INDEX_DIR / "vector_mapping.json"

# We use the base Legal-BERT, but wrapped in SentenceTransformer to enable pooling
# Alternatively, zlucia/custom-legalbert or standard nlpaueb/legal-bert-base-uncased
MODEL_NAME = "nlpaueb/legal-bert-base-uncased"

def get_device():
    if torch.backends.mps.is_available():
        return "mps"
    elif torch.cuda.is_available():
        return "cuda"
    return "cpu"

def build_index(limit=None, batch_size=32):
    print(f"Loading corpus from {CORPUS_PATH}...")
    with open(CORPUS_PATH) as f:
        data = json.load(f)
        
    documents = data.get("chunks", [])
    if not documents:
        print("No chunks found in corpus.")
        return

    # Extract all chunks
    chunks = []
    chunk_ids = []
    
    for doc_data in documents:
        text = doc_data.get("text", "")
        doc_id = doc_data.get("chunk_id", "")
        if text.strip() and doc_id:
            chunks.append(text)
            chunk_ids.append(doc_id)
            
    if limit:
        chunks = chunks[:limit]
        chunk_ids = chunk_ids[:limit]

    print(f"Found {len(chunks)} chunks to embed.")
    
    device = get_device()
    print(f"Loading model '{MODEL_NAME}' on {device}...")
    
    # We use SentenceTransformer to load the HuggingFace model and automatically add a pooling layer
    # so we get single vector representations for each chunk.
    model = SentenceTransformer(MODEL_NAME, device=device)
    
    print("Encoding texts... (this may take a while for large corpora)")
    start_time = time.time()
    
    # Generate embeddings
    embeddings = model.encode(chunks, batch_size=batch_size, show_progress_bar=True, convert_to_numpy=True)
    
    elapsed = time.time() - start_time
    print(f"Encoding complete in {elapsed:.2f} seconds ({len(chunks)/elapsed:.2f} chunks/sec).")
    
    # Ensure embeddings are float32 for FAISS
    embeddings = np.array(embeddings).astype("float32")
    
    # Build FAISS index
    dim = embeddings.shape[1]
    print(f"Building FAISS index with dimension {dim}...")
    
    # Using L2 distance, though Inner Product (Cosine Similarity) is also popular. 
    # For inner product, vectors should be normalized.
    faiss.normalize_L2(embeddings)
    index = faiss.IndexFlatIP(dim) 
    index.add(embeddings)
    
    # Save index and mapping
    faiss.write_index(index, str(FAISS_INDEX_PATH))
    
    # NEW: Save raw float32 binary for Rust Native ML integration
    raw_bin_path = INDEX_DIR / "vector_index.bin"
    with open(raw_bin_path, "wb") as f:
        f.write(embeddings.tobytes())
    
    with open(MAPPING_PATH, "w") as f:
        json.dump({"chunk_ids": chunk_ids, "dim": dim}, f)
        
    print(f"✅ Saved vector index to {FAISS_INDEX_PATH}")
    print(f"✅ Saved raw binary vectors to {raw_bin_path}")
    print(f"✅ Saved ID mapping to {MAPPING_PATH}")

def main():
    parser = argparse.ArgumentParser(description="Build Semantic Vector Index")
    parser.add_argument("--limit", type=int, help="Limit number of chunks for testing")
    parser.add_argument("--batch-size", type=int, default=32, help="Batch size for inference")
    args = parser.parse_args()
    
    build_index(limit=args.limit, batch_size=args.batch_size)

if __name__ == "__main__":
    main()
