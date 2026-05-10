from scripts.fetch_founders_online import (
    detect_waf_challenge,
    identifier_from_permalink,
    load_metadata_records,
    metadata_items,
    normalize_metadata_record,
)


class DummyResponse:
    status_code = 202
    headers = {"x-amzn-waf-action": "challenge"}


def test_identifier_from_permalink():
    assert (
        identifier_from_permalink("https://founders.archives.gov/documents/Franklin/01-04-02-0091")
        == "Franklin/01-04-02-0091"
    )


def test_normalize_metadata_record_derives_identifier():
    record = normalize_metadata_record(
        {
            "title": "Jonathan Belcher to Benjamin Franklin, 17 February 1752",
            "permalink": "https://founders.archives.gov/documents/Franklin/01-04-02-0091",
            "project": "Franklin Papers",
            "authors": ["Belcher, Jonathan"],
            "recipients": ["Franklin, Benjamin"],
            "date-from": "1752-02-17",
            "date-to": "1752-02-17",
        }
    )

    assert record is not None
    assert record.identifier == "Franklin/01-04-02-0091"
    assert record.authors == ["Belcher, Jonathan"]
    assert record.date_from == "1752-02-17"


def test_metadata_items_accepts_common_wrappers():
    assert metadata_items({"documents": [{"id": "A"}]}) == [{"id": "A"}]
    assert metadata_items([{"id": "B"}]) == [{"id": "B"}]


def test_primary_only_filters_editorial_records(tmp_path):
    metadata_path = tmp_path / "metadata.json"
    metadata_path.write_text(
        """
        [
          {
            "id": "Adams/01-01-02-0001",
            "title": "Dated primary document",
            "date-from": "1776-01-01"
          },
          {
            "id": "Adams/intro",
            "title": "Editorial introduction"
          }
        ]
        """,
        encoding="utf-8",
    )

    records = load_metadata_records(metadata_path, primary_only=True)

    assert [record.identifier for record in records] == ["Adams/01-01-02-0001"]


def test_detect_waf_challenge():
    assert detect_waf_challenge(DummyResponse())
