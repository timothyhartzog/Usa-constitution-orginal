"""Source URL validation for every chunk."""


def test_every_chunk_has_source_url(corpus):
    for chunk in corpus["chunks"]:
        assert chunk["source_url"], f"{chunk['chunk_id']} is missing source_url"
        assert chunk["source_url"].startswith(("http://", "https://")), f"{chunk['chunk_id']} has an invalid source_url"
