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
}
