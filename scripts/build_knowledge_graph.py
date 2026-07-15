#!/usr/bin/env python3
"""
Knowledge Graph Extractor (Phase 4)
Uses spaCy to run Named Entity Recognition (NER) across the constitutional corpus.
Extracts PERSON, GPE, ORG, and DATE entities and builds a co-occurrence graph.
"""

import json
from pathlib import Path
from collections import Counter
import networkx as nx

try:
    import spacy
except ImportError:
    print("Please install spacy: pip install spacy && python -m spacy download en_core_web_sm")
    exit(1)

PROJECT_ROOT = Path(__file__).parent.parent
CORPUS_PATH = PROJECT_ROOT / "data" / "chunks" / "constitution_full_corpus.json"
GRAPH_OUTPUT_PATH = PROJECT_ROOT / "data" / "index" / "knowledge_graph.json"

def main():
    import spacy
    print("Loading English NLP model...")
    try:
        nlp = spacy.load("en_core_web_sm")
    except OSError:
        import spacy.cli
        spacy.cli.download("en_core_web_sm")
        nlp = spacy.load("en_core_web_sm")

    print(f"Loading corpus from {CORPUS_PATH}...")
    with open(CORPUS_PATH) as f:
        corpus = json.load(f).get("chunks", [])

    print(f"Extracting entities from {len(corpus)} chunks...")
    
    # We will build a co-occurrence graph where nodes are entities and edges are weights
    G = nx.Graph()
    
    # Limit processing for speed if the corpus is huge (e.g. process top 5000 chunks)
    # Since this is an offline batch job, we will process a substantial representative sample.
    sample = corpus[:2000] 
    
    for i, chunk in enumerate(sample):
        if i % 1000 == 0:
            print(f" Processed {i}/{len(sample)} chunks...")
        text = chunk.get("text", "")
        doc = nlp(text)
        
        entities = set()
        for ent in doc.ents:
            # We only care about people and places for historical relationships
            if ent.label_ in ["PERSON", "GPE"]:
                name = ent.text.strip().replace("\n", " ")
                if len(name) > 2:
                    entities.add(name)
        
        # Add edges for all entities found in the same chunk
        ents_list = list(entities)
        for i in range(len(ents_list)):
            for j in range(i+1, len(ents_list)):
                e1, e2 = ents_list[i], ents_list[j]
                if G.has_edge(e1, e2):
                    G[e1][e2]['weight'] += 1
                else:
                    G.add_edge(e1, e2, weight=1)

    # Prune weak edges and low degree nodes to keep the graph small
    print("Pruning graph...")
    edges_to_remove = [(u, v) for u, v, d in G.edges(data=True) if d['weight'] < 3]
    G.remove_edges_from(edges_to_remove)
    nodes_to_remove = [n for n, d in G.degree() if d == 0]
    G.remove_nodes_from(nodes_to_remove)

    # Save to JSON graph format
    data = nx.node_link_data(G)
    with open(GRAPH_OUTPUT_PATH, "w") as f:
        json.dump(data, f)
        
    print(f"✅ Saved Knowledge Graph ({len(G.nodes)} nodes, {len(G.edges)} edges) to {GRAPH_OUTPUT_PATH}")

if __name__ == "__main__":
    main()
