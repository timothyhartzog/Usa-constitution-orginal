/**
 * Dashboard Controller
 *
 * Main dashboard logic and orchestration.
 */

class Dashboard {
    constructor() {
        this.loader = new AnalyticsLoader('/api');
        this.renderer = new ChartRenderer();
        this.loadingOverlay = document.getElementById('loading-overlay');
        this.errorMessage = document.getElementById('error-message');
        this.refreshBtn = document.getElementById('refresh-btn');
        this.filteredData = null;
        this.temporalData = null;

        this.attachEventListeners();
        this.init();
    }

    attachEventListeners() {
        this.refreshBtn.addEventListener('click', () => this.refresh());
        document.getElementById('apply-filters-btn').addEventListener('click', () => this.applyFilters());
        document.getElementById('reset-filters-btn').addEventListener('click', () => this.resetFilters());
        document.getElementById('compare-authors-btn').addEventListener('click', () => this.compareAuthors());
    }

    async init() {
        try {
            this.showLoading();
            await this.loadAndRender();
            this.hideLoading();
        } catch (error) {
            this.hideLoading();
            this.showError(`Failed to load analytics: ${error.message}`);
        }
    }

    async loadAndRender() {
        await this.loader.loadAllData();
        this.loader.populateFilterOptions();
        this.temporalData = await this.loader.loadTemporalData();
        this.renderAll();
    }

    renderAll() {
        this.loader.updateStatCards();

        const overview = this.loader.getOverview();
        const clauses = this.loader.getTopClauses();
        const issues = this.loader.getTopIssues();
        const collections = this.loader.getCollections();
        const authors = this.loader.getAuthors();
        const documents = this.loader.getDocuments();
        const words = this.loader.getWordFrequency();
        const chunkSizes = this.loader.getChunkSizes();
        const clauseIssueMatrix = this.loader.getClauseIssueMatrix();
        const authorClauseMatrix = this.loader.getAuthorClauseMatrix();
        const relationships = this.loader.getDocumentRelationships();

        this.renderer.renderTopClausesChart(clauses, 'chart-top-clauses');
        this.renderer.renderTopIssuesChart(issues, 'chart-top-issues');
        this.renderer.renderCollectionsPieChart(collections, 'chart-collections-pie');
        this.renderer.renderAuthorsPieChart(authors, 'chart-authors-pie');
        this.renderer.renderWordFrequencyChart(words, 'chart-word-frequency');
        this.renderer.renderChunkSizesChart(chunkSizes, 'chart-chunk-sizes');
        this.renderer.renderClauseIssueHeatmap(clauseIssueMatrix, 'chart-clause-issue-heatmap');
        this.renderer.renderAuthorClauseHeatmap(authorClauseMatrix, 'chart-author-clause-heatmap');
        this.renderer.renderDocumentNetworkGraph(relationships, 'chart-document-network');
        this.renderer.renderDocumentScatterChart(documents, 'chart-document-scatter');
        this.renderer.renderDocumentsBarChart(documents, 'chart-documents-bar');
        this.renderer.renderWordCloud(words, 'chart-word-cloud');

        if (this.temporalData && Object.keys(this.temporalData).length > 0) {
            this.renderer.renderTemporalChart(this.temporalData, 'chart-temporal');
        }
    }

    async applyFilters() {
        try {
            this.showLoading();
            const filters = this.loader.getActiveFilters();
            this.filteredData = await this.loader.loadFilteredData(filters);

            this.loader.corpus_overview = this.filteredData.corpus_overview;
            this.loader.clause_analysis = this.filteredData.clause_analysis;
            this.loader.issue_analysis = this.filteredData.issue_analysis;
            this.loader.collection_analysis = this.filteredData.collection_analysis;
            this.loader.author_analysis = this.filteredData.author_analysis;

            this.renderAll();
            this.hideLoading();
            this.showMessage('Filters applied successfully!');
        } catch (error) {
            this.hideLoading();
            this.showError(`Failed to apply filters: ${error.message}`);
        }
    }

    resetFilters() {
        this.loader.resetFilters();
        this.filteredData = null;
        this.loadAndRender();
        this.showMessage('Filters reset!');
    }

    async compareAuthors() {
        try {
            const author1 = document.getElementById('compare-author1').value;
            const author2 = document.getElementById('compare-author2').value;

            if (!author1 || !author2) {
                this.showError('Please select both authors to compare');
                return;
            }

            this.showLoading();
            const comparisonData = await this.loader.compareAuthors(author1, author2);
            this.hideLoading();

            this.renderer.renderAuthorComparisonChart(comparisonData, 'chart-author-comparison');

            const agreementScore = Math.round((comparisonData.agreement_score || 0) * 100);
            this.showMessage(`Agreement Score: ${agreementScore}%`);
        } catch (error) {
            this.hideLoading();
            this.showError(`Failed to compare authors: ${error.message}`);
        }
    }

    async refresh() {
        try {
            this.showLoading();
            await this.loadAndRender();
            this.hideLoading();
            this.showMessage('Analytics refreshed successfully!');
        } catch (error) {
            this.hideLoading();
            this.showError(`Failed to refresh analytics: ${error.message}`);
        }
    }

    showLoading() {
        this.loadingOverlay.classList.remove('hidden');
    }

    hideLoading() {
        this.loadingOverlay.classList.add('hidden');
    }

    showError(message) {
        this.errorMessage.textContent = message;
        this.errorMessage.classList.remove('hidden');
        setTimeout(() => this.errorMessage.classList.add('hidden'), 5000);
    }

    showMessage(message) {
        const msg = document.createElement('div');
        msg.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: #28a745;
            color: white;
            padding: 16px;
            border-radius: 4px;
            z-index: 3000;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        `;
        msg.textContent = message;
        document.body.appendChild(msg);

        setTimeout(() => msg.remove(), 3000);
    }
}

// Initialize dashboard when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    new Dashboard();
});
