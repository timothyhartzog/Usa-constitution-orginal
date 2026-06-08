"""Schema-level validation of `data/delegates.json`."""

import json
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).parent.parent

REQUIRED_FIELDS = {
    "id",
    "name",
    "state",
    "born",
    "died",
    "status",
    "notable_role",
    "archive_url",
}

KNOWN_STATUSES = {
    "signed",
    "signed_by_proxy",
    "refused_to_sign",
    "left_before_signing",
}

# The 12 states that sent delegates. (Rhode Island sent none.)
KNOWN_STATES = {
    "Connecticut",
    "Delaware",
    "Georgia",
    "Maryland",
    "Massachusetts",
    "New Hampshire",
    "New Jersey",
    "New York",
    "North Carolina",
    "Pennsylvania",
    "South Carolina",
    "Virginia",
}


@pytest.fixture(scope="module")
def delegates() -> list[dict]:
    payload = json.loads((PROJECT_ROOT / "data" / "delegates.json").read_text(encoding="utf-8"))
    return payload["delegates"]


@pytest.fixture(scope="module")
def manifest_doc_ids() -> set[str]:
    payload = json.loads(
        (PROJECT_ROOT / "config" / "sources_manifest.json").read_text(encoding="utf-8")
    )
    return {
        doc["document_id"]
        for collection in payload["collections"]
        for doc in collection["documents"]
    }


def test_total_count_is_55(delegates):
    assert len(delegates) == 55, f"expected 55 delegates, found {len(delegates)}"


def test_signing_breakdown_matches_history(delegates):
    counts = {s: 0 for s in KNOWN_STATUSES}
    for d in delegates:
        counts[d["status"]] = counts.get(d["status"], 0) + 1
    # 38 personally signed, plus Dickinson signed by proxy → 39 signers.
    assert counts["signed"] == 38, counts
    assert counts["signed_by_proxy"] == 1, counts
    assert counts["refused_to_sign"] == 3, counts
    assert counts["left_before_signing"] == 13, counts


def test_every_delegate_has_required_fields(delegates):
    for d in delegates:
        missing = REQUIRED_FIELDS.difference(d)
        assert not missing, f"{d.get('id')} missing fields {sorted(missing)}"


def test_ids_are_unique(delegates):
    ids = [d["id"] for d in delegates]
    assert len(ids) == len(set(ids)), f"duplicate delegate id(s): {sorted(set(i for i in ids if ids.count(i) > 1))}"


def test_states_are_known(delegates):
    for d in delegates:
        assert d["state"] in KNOWN_STATES, f"{d['id']} has unknown state {d['state']!r}"


def test_status_values_are_known(delegates):
    for d in delegates:
        assert d["status"] in KNOWN_STATUSES, f"{d['id']} has unknown status {d['status']!r}"


def test_archive_urls_are_http(delegates):
    for d in delegates:
        url = d["archive_url"]
        assert url.startswith(("http://", "https://")), f"{d['id']} archive_url not http(s): {url!r}"


def test_dates_are_iso_8601(delegates):
    import re
    iso = re.compile(r"^\d{4}-\d{2}-\d{2}$")
    for d in delegates:
        for field in ("born", "died"):
            assert iso.match(d[field]), f"{d['id']}.{field}={d[field]!r} is not ISO-8601"


def test_manifest_entry_references_exist(delegates, manifest_doc_ids):
    for d in delegates:
        manifest_entry = d.get("manifest_entry")
        if not manifest_entry:
            continue
        assert manifest_entry in manifest_doc_ids, (
            f"{d['id']}.manifest_entry={manifest_entry!r} is not a document_id in "
            f"sources_manifest.json"
        )


def test_at_least_one_signer_per_attending_state(delegates):
    by_state: dict[str, set[str]] = {}
    for d in delegates:
        by_state.setdefault(d["state"], set()).add(d["status"])
    missing = [s for s in KNOWN_STATES if s not in by_state]
    assert not missing, f"states without delegates: {missing}"
    # Every state that sent delegates produced at least one signer (signed or proxy)
    # except Maryland's status mix is OK because Carroll signed.
    no_signer = [
        state
        for state, statuses in by_state.items()
        if not statuses.intersection({"signed", "signed_by_proxy"})
    ]
    assert not no_signer, f"states with no signers: {no_signer}"
