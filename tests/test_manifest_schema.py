"""Schema-level validation of `config/sources_manifest.json`.

These tests run regardless of whether the corpus has been ingested, so they
catch typos, duplicate identifiers, and unknown chunk strategies before any
network round-trip.
"""

REQUIRED_DOC_FIELDS = {
    "document_id",
    "title",
    "author",
    "date",
    "document_type",
    "source_url",
    "source_format",
    "chunk_strategy",
    "default_issue_tags",
}

KNOWN_CHUNK_STRATEGIES = {
    "constitution_sections",
    "federalist_essays",
    "jefferson_letters",
    "numbered_essay_series",
    "correspondence_letters",
    "sliding_window",
}

KNOWN_SOURCE_FORMATS = {"text", "html"}


def test_manifest_has_collections(sources_manifest):
    assert sources_manifest["collections"], "manifest has no collections"


def test_every_document_has_required_fields(sources_manifest):
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            missing = REQUIRED_DOC_FIELDS.difference(document)
            assert not missing, (
                f"{collection['collection_id']}/{document.get('document_id', '<missing>')} "
                f"is missing fields {sorted(missing)}"
            )


def test_document_ids_are_unique(sources_manifest):
    seen: set[str] = set()
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            doc_id = document["document_id"]
            assert doc_id not in seen, f"duplicate document_id: {doc_id}"
            seen.add(doc_id)


def test_chunk_strategies_are_known(sources_manifest):
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            strategy = document["chunk_strategy"]
            assert strategy in KNOWN_CHUNK_STRATEGIES, (
                f"{document['document_id']} uses unknown chunk_strategy={strategy!r}; "
                f"add it to chunk_documents.build_records or KNOWN_CHUNK_STRATEGIES."
            )


def test_source_formats_are_known(sources_manifest):
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            fmt = document["source_format"]
            assert fmt in KNOWN_SOURCE_FORMATS, (
                f"{document['document_id']} uses unknown source_format={fmt!r}"
            )


def test_default_issue_tags_are_lists_of_strings(sources_manifest):
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            tags = document["default_issue_tags"]
            assert isinstance(tags, list)
            for tag in tags:
                assert isinstance(tag, str) and tag, (
                    f"{document['document_id']} has invalid default_issue_tag {tag!r}"
                )


def test_chunk_options_only_attached_to_supported_strategies(sources_manifest):
    supported = {"numbered_essay_series", "correspondence_letters"}
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            if "chunk_options" in document:
                assert document["chunk_strategy"] in supported, (
                    f"{document['document_id']} sets chunk_options but uses "
                    f"unsupported strategy {document['chunk_strategy']!r}"
                )


def test_source_urls_are_https_or_http(sources_manifest):
    for collection in sources_manifest["collections"]:
        for document in collection["documents"]:
            url = document["source_url"]
            assert url.startswith(("http://", "https://")), (
                f"{document['document_id']} has non-http source_url: {url!r}"
            )
