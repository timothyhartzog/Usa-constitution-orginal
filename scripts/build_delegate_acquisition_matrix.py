#!/usr/bin/env python3
"""Build a prioritized acquisition matrix for delegates' own writings.

Joins the real per-delegate dossiers (authored vs. mentioned counts, already
committed under data/delegates/dossiers/) with the curated public-domain source
targets in config/delegate_acquisition_targets.json, then classifies every one
of the 55 Federal Convention delegates by how complete their *authored* corpus
is and what to fetch next.

This script is fully offline: it reads only committed JSON dossiers, so it runs
even when the network is blocked and the large LFS chunk corpus is unavailable.

Outputs:
  data/delegates/reports/acquisition_matrix.csv
  data/delegates/reports/acquisition_matrix.json
"""
from __future__ import annotations

import csv
import json
import re
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).resolve().parent.parent
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from scripts.utils.pipeline import DATA_DIR, ensure_directory, load_json, save_json

DOSSIER_DIR = DATA_DIR / "delegates" / "dossiers"
REPORT_DIR = DATA_DIR / "delegates" / "reports"
DELEGATES_PATH = DATA_DIR / "delegates" / "federal_convention_delegates.json"
TARGETS_PATH = PROJECT_ROOT / "config" / "delegate_acquisition_targets.json"

# Authored-passage thresholds for the "authored corpus" classification.
AUTHORED_GOOD = 100   # a substantial body of the delegate's own writing is loaded
AUTHORED_SOME = 5     # a token amount of authored material is present


def slug(name: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", name.lower()).strip("_")


def classify(authored: int, survival: str) -> str:
    """Coverage status for a delegate's OWN writings."""
    if authored >= AUTHORED_GOOD:
        return "authored-good"
    if authored >= AUTHORED_SOME:
        return "authored-partial"
    # No real authored corpus loaded yet.
    if survival == "minimal":
        return "minimal-surviving"
    return "mentions-only"


def main() -> None:
    delegates = load_json(DELEGATES_PATH)["delegates"]
    targets = {t["name"]: t for t in load_json(TARGETS_PATH)["targets"]}

    rows: list[dict[str, object]] = []
    for delegate in delegates:
        name = delegate["name"]
        dossier_path = DOSSIER_DIR / f"{slug(name)}.json"
        if dossier_path.exists():
            cov = load_json(dossier_path)["coverage"]
            authored = int(cov.get("authored_chunks", 0))
            mentions = int(cov.get("matching_chunks", 0))
            total_mentions = int(cov.get("total_mentions", 0))
        else:
            authored = mentions = total_mentions = 0

        target = targets.get(name, {})
        survival = str(target.get("survival", "unknown"))
        collision = bool(target.get("collision_word", False))
        have_corpus = bool(target.get("have_authored_corpus", False))
        ia = list(target.get("internet_archive", []))

        status = classify(authored, survival)

        # Priority: what gives the biggest authored-coverage gain per delegate.
        # 1 = rich surviving papers, PD edition identified, not yet loaded.
        if status in {"authored-good"} or have_corpus:
            priority = 0  # already covered
        elif survival == "rich" and ia:
            priority = 1
        elif survival in {"rich", "moderate"} and ia:
            priority = 2
        elif survival in {"rich", "moderate"}:
            priority = 3  # PD source named but no IA id yet (needs locating)
        else:
            priority = 4  # minimal surviving papers; best-effort only

        rows.append(
            {
                "name": name,
                "state": delegate["state"],
                "signed": delegate["signed_constitution"],
                "authored_chunks": authored,
                "mention_chunks": mentions,
                "total_mentions": total_mentions,
                "collision_word": collision,
                "survival": survival,
                "status": status,
                "priority": priority,
                "have_authored_corpus": have_corpus,
                "internet_archive_ids": ";".join(ia),
                "pd_source": str(target.get("pd_source", "")),
            }
        )

    rows.sort(key=lambda r: (r["priority"], -int(r["authored_chunks"]), str(r["name"])))

    ensure_directory(REPORT_DIR)
    with (REPORT_DIR / "acquisition_matrix.csv").open("w", encoding="utf-8", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        writer.writerows(rows)

    # Summary counts.
    by_status: dict[str, int] = {}
    by_priority: dict[int, int] = {}
    for r in rows:
        by_status[r["status"]] = by_status.get(r["status"], 0) + 1
        by_priority[r["priority"]] = by_priority.get(r["priority"], 0) + 1

    save_json(
        REPORT_DIR / "acquisition_matrix.json",
        {
            "metadata": {
                "delegate_count": len(rows),
                "authored_good_threshold": AUTHORED_GOOD,
                "authored_some_threshold": AUTHORED_SOME,
                "status_counts": by_status,
                "priority_counts": {str(k): v for k, v in sorted(by_priority.items())},
            },
            "delegates": rows,
        },
    )

    print(f"Delegates classified: {len(rows)}")
    print("Status counts:")
    for status, count in sorted(by_status.items(), key=lambda kv: -kv[1]):
        print(f"  {status:20} {count}")
    print("Priority counts (0=covered .. 4=minimal/best-effort):")
    for prio in sorted(by_priority):
        print(f"  P{prio}: {by_priority[prio]}")
    print(f"Saved {REPORT_DIR / 'acquisition_matrix.csv'}")
    print(f"Saved {REPORT_DIR / 'acquisition_matrix.json'}")


if __name__ == "__main__":
    main()
