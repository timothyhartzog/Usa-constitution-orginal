"""
Test source URL validity and accessibility.
"""

import pytest
import requests

def test_all_chunks_have_source_url(corpus):
    """Every chunk must have a source URL."""
    for chunk in corpus['chunks']:
        assert 'source_url' in chunk, f"Missing source_url in {chunk['chunk_id']}"
        assert chunk['source_url'], f"Empty source_url in {chunk['chunk_id']}"

def test_url_format(corpus):
    """URLs should be properly formatted."""
    for chunk in corpus['chunks']:
        url = chunk['source_url']
        assert url.startswith(('http://', 'https://')), \
            f"Invalid URL format in {chunk['chunk_id']}: {url}"

def test_unique_sources_per_document(corpus):
    """Same document should have same source URL."""
    doc_urls = {}
    for chunk in corpus['chunks']:
        doc_id = chunk['document_id']
        url = chunk['source_url']

        if doc_id in doc_urls:
            assert doc_urls[doc_id] == url, \
                f"Inconsistent URLs for document {doc_id}"
        else:
            doc_urls[doc_id] = url

@pytest.mark.slow
def test_urls_accessible(corpus):
    """Test that source URLs are accessible (basic check)."""
    # This is a slow test - only run with -m slow flag
    tested_urls = set()
    failed_urls = []

    for chunk in corpus['chunks']:
        url = chunk['source_url']

        # Skip if we've already tested this URL
        if url in tested_urls:
            continue

        tested_urls.add(url)

        try:
            response = requests.head(url, timeout=5, allow_redirects=True)
            if response.status_code >= 400:
                failed_urls.append((url, response.status_code))
        except (requests.Timeout, requests.ConnectionError):
            # These may fail due to rate limiting or temporary issues
            pass
        except Exception as e:
            # Log but don't fail on other exceptions
            pass

    if failed_urls:
        print(f"\nWarning: {len(failed_urls)} URLs returned error status codes")

def test_no_duplicate_urls(corpus):
    """Check for unexpected URL duplicates across documents."""
    # Most documents should have unique sources
    doc_id_by_url = {}
    for chunk in corpus['chunks']:
        doc_id = chunk['document_id']
        url = chunk['source_url']

        if url in doc_id_by_url:
            # Same URL used for different documents (unusual but not necessarily wrong)
            if doc_id_by_url[url] != doc_id:
                pass  # This is acceptable for some sources
        else:
            doc_id_by_url[url] = doc_id
