"""Uniqueness checks for chunk identifiers."""


def test_chunk_id_is_unique(corpus):
    chunk_ids = [chunk["chunk_id"] for chunk in corpus["chunks"]]
    assert len(chunk_ids) == len(set(chunk_ids)), "chunk_id values must be unique"
