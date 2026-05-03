/**
 * Analytics Loader
 *
 * Loads analytics data from Flask API endpoints and manages data state.
 */

class AnalyticsLoader {
    constructor(apiBaseUrl = '/api') {
        this.apiBaseUrl = apiBaseUrl;
        this.data = {};
        this.lastUpdated = null;
    }

    async loadAllData() {
        try {
            const response = await fetch(`${this.apiBaseUrl}/analytics/all`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);

            this.data = await response.json();
            this.lastUpdated = new Date();
            return this.data;
        } catch (error) {
            console.error('Failed to load analytics data:', error);
            throw error;
        }
    }

    async loadEndpoint(endpoint) {
        try {
            const response = await fetch(`${this.apiBaseUrl}/analytics/${endpoint}`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            console.error(`Failed to load ${endpoint}:`, error);
            throw error;
        }
    }

    getOverview() {
        return this.data.corpus_overview || {};
    }

    getTopClauses() {
        const clauses = this.data.clause_analysis || {};
        return clauses.top_clauses || [];
    }

    getTopIssues() {
        const issues = this.data.issue_analysis || {};
        return issues.top_issues || [];
    }

    getCollections() {
        return this.data.collection_analysis || [];
    }

    getAuthors() {
        return this.data.author_analysis || [];
    }

    getDocuments() {
        return this.data.document_analysis || [];
    }

    getWordFrequency() {
        const words = this.data.word_analysis || {};
        return words.top_words || [];
    }

    getChunkSizes() {
        return this.data.chunk_size_distribution || {};
    }

    getClauseIssueMatrix() {
        return this.data.clause_issue_matrix || {};
    }

    getAuthorClauseMatrix() {
        return this.data.author_clause_matrix || {};
    }

    getDocumentRelationships() {
        return this.data.document_relationships || [];
    }

    updateStatCards() {
        const overview = this.getOverview();

        document.getElementById('stat-chunks').textContent =
            (overview.total_chunks || 0).toLocaleString();
        document.getElementById('stat-documents').textContent =
            (overview.total_documents || 0).toLocaleString();
        document.getElementById('stat-words').textContent =
            (overview.total_words || 0).toLocaleString();
        document.getElementById('stat-avg-chunk').textContent =
            Math.round(overview.average_chunk_size || 0);
        document.getElementById('stat-clauses').textContent =
            (overview.total_clause_tags || 0).toLocaleString();
        document.getElementById('stat-issues').textContent =
            (overview.total_issue_tags || 0).toLocaleString();

        if (this.lastUpdated) {
            document.getElementById('last-updated').textContent =
                `Last updated: ${this.lastUpdated.toLocaleString()}`;
        }
    }

    getFormattedLastUpdated() {
        return this.lastUpdated ? this.lastUpdated.toLocaleString() : 'Never';
    }

    async loadFilteredData(filters) {
        try {
            const response = await fetch(`${this.apiBaseUrl}/analytics/filtered`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(filters)
            });
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            console.error('Failed to load filtered analytics:', error);
            throw error;
        }
    }

    async loadTemporalData() {
        try {
            return await this.loadEndpoint('temporal');
        } catch (error) {
            console.error('Failed to load temporal data:', error);
            return {};
        }
    }

    async compareAuthors(author1, author2, clause = null) {
        try {
            const response = await fetch(`${this.apiBaseUrl}/analytics/author-comparison`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ author1, author2, clause })
            });
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            return await response.json();
        } catch (error) {
            console.error('Failed to compare authors:', error);
            throw error;
        }
    }

    async generateReport() {
        try {
            return await this.loadEndpoint('report');
        } catch (error) {
            console.error('Failed to generate report:', error);
            return {};
        }
    }

    populateFilterOptions() {
        const collections = new Set();
        const authors = new Set();

        const collectionAnalysis = this.getCollections();
        collectionAnalysis.forEach(c => collections.add(c.collection));

        const authorAnalysis = this.getAuthors();
        authorAnalysis.forEach(a => authors.add(a.author));

        const collectionSelect = document.getElementById('filter-collections');
        if (collectionSelect) {
            collections.forEach(coll => {
                const option = document.createElement('option');
                option.value = coll;
                option.textContent = coll.replace(/_/g, ' ');
                collectionSelect.appendChild(option);
            });
        }

        const authorSelect = document.getElementById('filter-authors');
        if (authorSelect) {
            authors.forEach(author => {
                const option = document.createElement('option');
                option.value = author;
                option.textContent = author;
                authorSelect.appendChild(option);
            });
        }

        const compareAuthor1 = document.getElementById('compare-author1');
        const compareAuthor2 = document.getElementById('compare-author2');
        if (compareAuthor1 && compareAuthor2) {
            authors.forEach(author => {
                [compareAuthor1, compareAuthor2].forEach(select => {
                    const option = document.createElement('option');
                    option.value = author;
                    option.textContent = author;
                    select.appendChild(option);
                });
            });
        }
    }

    getActiveFilters() {
        const filters = {};

        const collectionSelect = document.getElementById('filter-collections');
        if (collectionSelect) {
            const selected = Array.from(collectionSelect.selectedOptions).map(o => o.value).filter(v => v);
            if (selected.length > 0) filters.collections = selected;
        }

        const authorSelect = document.getElementById('filter-authors');
        if (authorSelect) {
            const selected = Array.from(authorSelect.selectedOptions).map(o => o.value).filter(v => v);
            if (selected.length > 0) filters.authors = selected;
        }

        const startDate = document.getElementById('filter-start-date');
        if (startDate && startDate.value) filters.start_date = startDate.value;

        const endDate = document.getElementById('filter-end-date');
        if (endDate && endDate.value) filters.end_date = endDate.value;

        return filters;
    }

    resetFilters() {
        document.getElementById('filter-collections').selectedIndex = 0;
        document.getElementById('filter-authors').selectedIndex = 0;
        document.getElementById('filter-start-date').value = '';
        document.getElementById('filter-end-date').value = '';
    }
}
