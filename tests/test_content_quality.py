"""Basic content quality checks for chunk text."""


def test_chunk_text_is_not_empty(corpus):
    for chunk in corpus["chunks"]:
        assert chunk["text"].strip(), f"{chunk['chunk_id']} has empty text"


def test_word_count_matches_text(corpus):
    for chunk in corpus["chunks"]:
        assert chunk["word_count"] == len(chunk["text"].split()), f"{chunk['chunk_id']} has an incorrect word_count"
