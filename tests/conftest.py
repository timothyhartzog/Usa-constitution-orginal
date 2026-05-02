"""
Shared test fixtures for constitutional research system.
"""

import json
import pytest
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent

@pytest.fixture
def corpus():
    """Load full corpus for testing."""
    corpus_path = PROJECT_ROOT / "data" / "chunks" / "constitution_full_corpus.json"
    if not corpus_path.exists():
        pytest.skip("Corpus not generated yet")

    with open(corpus_path) as f:
        return json.load(f)

@pytest.fixture
def search_index():
    """Load search index for testing."""
    index_path = PROJECT_ROOT / "data" / "index" / "search_index.json"
    if not index_path.exists():
        pytest.skip("Search index not generated yet")

    with open(index_path) as f:
        return json.load(f)

@pytest.fixture
def sources_manifest():
    """Load sources manifest."""
    manifest_path = PROJECT_ROOT / "config" / "sources_manifest.json"
    with open(manifest_path) as f:
        return json.load(f)

@pytest.fixture
def constitutional_clauses():
    """Load constitutional clauses configuration."""
    clauses_path = PROJECT_ROOT / "config" / "constitutional_clauses.json"
    with open(clauses_path) as f:
        return json.load(f)
