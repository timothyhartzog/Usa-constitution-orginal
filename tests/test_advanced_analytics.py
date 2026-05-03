#!/usr/bin/env python3
"""
Tests for advanced analytics features: filtering, temporal analysis, and comparison.
"""

import pytest
from pathlib import Path
from scripts.analytics_engine import CorpusAnalytics


@pytest.fixture
def analytics():
    """Load analytics engine."""
    return CorpusAnalytics()


class TestFilteredAnalytics:
    """Test filtered analytics functionality."""

    def test_filter_by_collection(self, analytics):
        """Test filtering by collection."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        collections = set(c['collection'] for c in analytics.chunks)
        if collections:
            test_collection = list(collections)[0]
            result = analytics.analyze_with_filters(collections=[test_collection])

            assert 'corpus_overview' in result
            assert 'clause_analysis' in result
            assert result['corpus_overview']['total_chunks'] > 0

    def test_filter_by_author(self, analytics):
        """Test filtering by author."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        authors = set(c['author'] for c in analytics.chunks)
        if authors:
            test_author = list(authors)[0]
            result = analytics.analyze_with_filters(authors=[test_author])

            assert result['corpus_overview']['total_chunks'] > 0

    def test_filter_by_multiple_collections(self, analytics):
        """Test filtering by multiple collections."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        collections = list(set(c['collection'] for c in analytics.chunks))
        if len(collections) >= 2:
            result = analytics.analyze_with_filters(collections=collections[:2])
            assert result['corpus_overview']['total_chunks'] > 0

    def test_filter_by_date_range(self, analytics):
        """Test filtering by date range."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        dates = [c.get('date') for c in analytics.chunks if c.get('date')]
        if dates:
            dates.sort()
            start_date = dates[0]
            end_date = dates[len(dates)//2] if len(dates) > 1 else dates[0]

            result = analytics.analyze_with_filters(
                start_date=start_date,
                end_date=end_date
            )
            assert result['corpus_overview']['total_chunks'] > 0

    def test_combined_filters(self, analytics):
        """Test combining multiple filters."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        collections = list(set(c['collection'] for c in analytics.chunks))
        authors = list(set(c['author'] for c in analytics.chunks))

        if collections and authors:
            result = analytics.analyze_with_filters(
                collections=collections[:1],
                authors=authors[:1]
            )
            assert 'corpus_overview' in result


class TestTemporalAnalysis:
    """Test temporal analysis functionality."""

    def test_temporal_analysis_structure(self, analytics):
        """Test temporal analysis returns correct structure."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        result = analytics.analyze_temporal()

        assert isinstance(result, list)
        if result:
            first_item = result[0]
            assert 'date' in first_item
            assert 'clause' in first_item
            assert 'mentions' in first_item

    def test_temporal_analysis_sorting(self, analytics):
        """Test temporal analysis is sorted by date."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        result = analytics.analyze_temporal()

        if len(result) > 1:
            dates = [item['date'] for item in result]
            assert dates == sorted(dates)

    def test_temporal_analysis_mentions_positive(self, analytics):
        """Test all mentions are positive numbers."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        result = analytics.analyze_temporal()

        for item in result:
            assert item['mentions'] > 0


class TestAuthorComparison:
    """Test author comparison functionality."""

    def test_author_comparison_structure(self, analytics):
        """Test author comparison returns correct structure."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        authors = list(set(c['author'] for c in analytics.chunks))
        if len(authors) >= 2:
            result = analytics.compare_authors(authors[0], authors[1])

            assert 'author1' in result
            assert 'author2' in result
            assert 'shared_clauses' in result
            assert 'shared_issues' in result
            assert 'agreement_score' in result

    def test_author_comparison_stats(self, analytics):
        """Test author comparison stats are present."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        authors = list(set(c['author'] for c in analytics.chunks))
        if len(authors) >= 2:
            result = analytics.compare_authors(authors[0], authors[1])

            assert 'name' in result['author1']
            assert 'chunks' in result['author1']
            assert 'words' in result['author1']
            assert 'top_clauses' in result['author1']
            assert 'top_issues' in result['author1']

    def test_author_comparison_agreement_score(self, analytics):
        """Test agreement score is reasonable."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        authors = list(set(c['author'] for c in analytics.chunks))
        if len(authors) >= 2:
            result = analytics.compare_authors(authors[0], authors[1])

            score = result['agreement_score']
            assert 0 <= score <= 1

    def test_author_comparison_same_author(self, analytics):
        """Test comparing an author with itself."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        authors = list(set(c['author'] for c in analytics.chunks))
        if authors:
            result = analytics.compare_authors(authors[0], authors[0])

            assert len(result['shared_clauses']) > 0
            assert result['agreement_score'] == 1.0


class TestReportGeneration:
    """Test report generation."""

    def test_report_generation_structure(self, analytics):
        """Test report generation returns correct structure."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        report = analytics.generate_report()

        assert 'title' in report
        assert 'generated_at' in report
        assert 'executive_summary' in report
        assert 'key_findings' in report

    def test_executive_summary(self, analytics):
        """Test executive summary contains metrics."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        report = analytics.generate_report()
        summary = report['executive_summary']

        assert 'total_chunks' in summary
        assert 'total_documents' in summary
        assert 'total_words' in summary
        assert summary['total_chunks'] > 0

    def test_key_findings(self, analytics):
        """Test key findings are present."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        report = analytics.generate_report()
        findings = report['key_findings']

        assert 'most_discussed_clause' in findings
        assert 'most_discussed_issue' in findings
        assert 'clause_coverage' in findings
        assert 'issue_coverage' in findings

    def test_report_includes_statistics(self, analytics):
        """Test report includes document and author statistics."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        report = analytics.generate_report()

        assert 'document_statistics' in report
        assert 'collection_statistics' in report
        assert 'author_statistics' in report


class TestIntegration:
    """Integration tests for advanced features."""

    def test_filter_then_compare(self, analytics):
        """Test filtering followed by author comparison."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        collections = list(set(c['collection'] for c in analytics.chunks))
        if collections:
            filtered = analytics.analyze_with_filters(collections=collections[:1])

            assert 'author_analysis' in filtered
            assert len(filtered['author_analysis']) > 0

    def test_temporal_with_filtered_data(self, analytics):
        """Test temporal analysis works with filters applied."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        original_chunks = analytics.chunks

        collections = list(set(c['collection'] for c in analytics.chunks))
        if collections:
            analytics.chunks = [c for c in analytics.chunks
                                if c['collection'] == collections[0]]

            result = analytics.analyze_temporal()

            assert isinstance(result, list)

            analytics.chunks = original_chunks

    def test_all_analyses_without_error(self, analytics):
        """Test all advanced analytics complete without errors."""
        if not analytics.chunks:
            pytest.skip("No corpus data available")

        try:
            filtered = analytics.analyze_with_filters()
            temporal = analytics.analyze_temporal()
            authors = list(set(c['author'] for c in analytics.chunks))

            if len(authors) >= 2:
                comparison = analytics.compare_authors(authors[0], authors[1])

            report = analytics.generate_report()

            assert all([filtered, temporal, report])
        except Exception as e:
            pytest.fail(f"Advanced analytics failed: {str(e)}")


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
