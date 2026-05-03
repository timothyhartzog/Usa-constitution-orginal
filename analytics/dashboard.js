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

        this.attachEventListeners();
        this.init();
    }

    attachEventListeners() {
        this.refreshBtn.addEventListener('click', () => this.refresh());
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
