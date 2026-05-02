"""
Test content quality and validity.
"""

import pytest

def test_no_empty_chunks(corpus):
    """No chunk should have empty text."""
    for chunk in corpus['chunks']:
        assert len(chunk['text'].strip()) > 0, f"Empty chunk: {chunk['chunk_id']}"

def test_minimum_chunk_size(corpus):
    """All chunks should exceed minimum word count."""
    MIN_WORDS = 10
    for chunk in corpus['chunks']:
        assert chunk['word_count'] >= MIN_WORDS, \
            f"Chunk {chunk['chunk_id']} too small: {chunk['word_count']} words"

def test_maximum_chunk_size(corpus):
    """All chunks should not exceed maximum word count."""
    MAX_WORDS = 1000
    for chunk in corpus['chunks']:
        assert chunk['word_count'] <= MAX_WORDS, \
            f"Chunk {chunk['chunk_id']} too large: {chunk['word_count']} words"

def test_typical_chunk_size_distribution(corpus):
    """Most chunks should be in reasonable size range."""
    TARGET_MIN = 200
    TARGET_MAX = 600
    in_range = sum(1 for c in corpus['chunks'] if TARGET_MIN <= c['word_count'] <= TARGET_MAX)
    pct = in_range / len(corpus['chunks']) * 100
    assert pct >= 70, \
        f"Only {pct:.1f}% of chunks in ideal size range ({TARGET_MIN}-{TARGET_MAX} words)"

def test_valid_utf8_encoding(corpus):
    """All text should be valid UTF-8."""
    for chunk in corpus['chunks']:
        try:
            chunk['text'].encode('utf-8')
        except UnicodeEncodeError as e:
            pytest.fail(f"Invalid UTF-8 in chunk {chunk['chunk_id']}: {e}")

def test_clause_tag_validity(corpus, constitutional_clauses):
    """Constitutional clause tags should exist in taxonomy."""
    valid_clauses = {c['clause_id'] for c in constitutional_clauses['constitutional_clauses']}

    for chunk in corpus['chunks']:
        tags = chunk.get('constitutional_clause_tags', [])
        for tag in tags:
            assert tag in valid_clauses, \
                f"Invalid clause tag '{tag}' in chunk {chunk['chunk_id']}"

def test_issue_tag_validity(corpus, constitutional_clauses):
    """Issue tags should exist in taxonomy."""
    valid_issues = {t['tag_id'] for t in constitutional_clauses['issue_tags']}

    for chunk in corpus['chunks']:
        tags = chunk.get('issue_tags', [])
        for tag in tags:
            assert tag in valid_issues, \
                f"Invalid issue tag '{tag}' in chunk {chunk['chunk_id']}"

def test_no_excessive_whitespace(corpus):
    """Chunks should not have excessive consecutive whitespace."""
    import re
    for chunk in corpus['chunks']:
        # Check for more than 2 consecutive newlines
        if re.search(r'\n{3,}', chunk['text']):
            pytest.fail(f"Excessive newlines in chunk {chunk['chunk_id']}")

def test_collection_consistency(corpus):
    """All chunks in corpus should reference valid collections."""
    valid_collections = set(c['collection'] for c in corpus['chunks'])

    for chunk in corpus['chunks']:
        assert chunk['collection'] in valid_collections

def test_document_type_validity(corpus):
    """Document types should be reasonable values."""
    valid_types = {
        'foundational_document',
        'convention_notes',
        'convention_records',
        'political_essay',
        'correspondence',
        'document'
    }

    for chunk in corpus['chunks']:
        doc_type = chunk.get('document_type')
        assert doc_type in valid_types, \
            f"Invalid document_type '{doc_type}' in chunk {chunk['chunk_id']}"

def test_tags_are_lists(corpus):
    """Tag fields should always be lists."""
    for chunk in corpus['chunks']:
        assert isinstance(chunk.get('constitutional_clause_tags'), list)
        assert isinstance(chunk.get('issue_tags'), list)
