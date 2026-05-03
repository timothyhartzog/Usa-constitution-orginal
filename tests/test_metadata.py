"""Metadata validation for chunked corpus output."""


REQUIRED_FIELDS = {
    "chunk_id",
    "document_id",
    "title",
    "author",
    "date",
    "source_collection",
    "source_url",
    "document_type",
    "issue_tags",
    "constitutional_clause_tags",
    "text",
    "word_count",
}


def test_every_chunk_has_required_metadata(corpus):
    for chunk in corpus["chunks"]:
        missing = REQUIRED_FIELDS.difference(chunk)
        assert not missing, f"{chunk.get('chunk_id', '<missing>')} is missing {sorted(missing)}"


def test_metadata_field_types_are_consistent(corpus):
    for chunk in corpus["chunks"]:
        assert isinstance(chunk["issue_tags"], list)
        assert isinstance(chunk["constitutional_clause_tags"], list)
        assert isinstance(chunk["word_count"], int)
