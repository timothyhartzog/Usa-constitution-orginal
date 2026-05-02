"""
Test chunk metadata completeness and structure.
"""

import pytest

def test_chunk_has_required_fields(corpus):
    """All chunks must have required metadata fields."""
    required_fields = [
        'chunk_id', 'document_id', 'collection', 'title', 'author',
        'date', 'document_type', 'source_url', 'text', 'word_count',
        'constitutional_clause_tags', 'issue_tags'
    ]

    for chunk in corpus['chunks']:
        for field in required_fields:
            assert field in chunk, f"Missing field '{field}' in chunk {chunk.get('chunk_id')}"

def test_chunk_id_format(corpus):
    """Chunk IDs should follow format: collection_document_id_index."""
    for chunk in corpus['chunks']:
        chunk_id = chunk['chunk_id']
        parts = chunk_id.split('_')
        assert len(parts) >= 3, f"Invalid chunk_id format: {chunk_id}"

def test_chunk_id_uniqueness(corpus):
    """No duplicate chunk IDs."""
    chunk_ids = [c['chunk_id'] for c in corpus['chunks']]
    assert len(chunk_ids) == len(set(chunk_ids)), "Duplicate chunk IDs found"

def test_word_count_matches_text(corpus):
    """Word count field should match actual text word count."""
    for chunk in corpus['chunks']:
        actual_count = len(chunk['text'].split())
        recorded_count = chunk['word_count']
        assert actual_count == recorded_count, \
            f"Word count mismatch in {chunk['chunk_id']}: expected {actual_count}, got {recorded_count}"

def test_metadata_consistency(corpus):
    """Metadata fields should be consistent types."""
    for chunk in corpus['chunks']:
        assert isinstance(chunk['title'], str), f"title must be string in {chunk['chunk_id']}"
        assert isinstance(chunk['author'], str), f"author must be string in {chunk['chunk_id']}"
        assert isinstance(chunk['text'], str), f"text must be string in {chunk['chunk_id']}"
        assert isinstance(chunk['word_count'], int), f"word_count must be int in {chunk['chunk_id']}"
        assert isinstance(chunk['constitutional_clause_tags'], list), f"constitutional_clause_tags must be list in {chunk['chunk_id']}"
        assert isinstance(chunk['issue_tags'], list), f"issue_tags must be list in {chunk['chunk_id']}"

def test_tag_fields_are_lists(corpus):
    """Tags should be lists."""
    for chunk in corpus['chunks']:
        assert isinstance(chunk.get('constitutional_clause_tags', []), list)
        assert isinstance(chunk.get('issue_tags', []), list)

def test_corpus_metadata(corpus):
    """Corpus metadata should be present and valid."""
    assert 'metadata' in corpus
    assert corpus['metadata']['version'] == "1.0"
    assert corpus['metadata']['total_chunks'] == len(corpus['chunks'])
