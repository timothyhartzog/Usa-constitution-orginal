"""
Test chunk uniqueness and deduplication.
"""

import pytest

def test_chunk_id_uniqueness(corpus):
    """Every chunk should have a unique chunk_id."""
    chunk_ids = [c['chunk_id'] for c in corpus['chunks']]
    assert len(chunk_ids) == len(set(chunk_ids)), \
        f"Found {len(chunk_ids) - len(set(chunk_ids))} duplicate chunk IDs"

def test_document_id_within_collection(corpus):
    """Document IDs should be unique within each collection."""
    for collection in set(c['collection'] for c in corpus['chunks']):
        collection_chunks = [c for c in corpus['chunks'] if c['collection'] == collection]
        doc_ids = [c['document_id'] for c in collection_chunks]
        assert len(doc_ids) == len(set(doc_ids)), \
            f"Duplicate document IDs in collection {collection}"

def test_no_identical_chunks(corpus):
    """No two chunks should have identical text."""
    texts = {}
    duplicates = []

    for chunk in corpus['chunks']:
        text = chunk['text']
        if text in texts:
            duplicates.append((texts[text]['chunk_id'], chunk['chunk_id']))
        else:
            texts[text] = chunk

    assert len(duplicates) == 0, \
        f"Found {len(duplicates)} duplicate text chunks"

def test_no_empty_text_chunks(corpus):
    """No chunk should have empty text."""
    for chunk in corpus['chunks']:
        assert chunk['text'], f"Empty text in chunk {chunk['chunk_id']}"
        assert chunk['text'].strip(), f"Only whitespace in chunk {chunk['chunk_id']}"
