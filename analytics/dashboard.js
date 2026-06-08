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
        document.getElementById('download-report-json').addEventListener('click', () => this.downloadJSONReport());
        document.getElementById('download-report-html').addEventListener('click', () => this.downloadHTMLReport());
        document.getElementById('print-report-btn').addEventListener('click', () => this.printReport());
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

    downloadJSONReport() {
        const report = {
            title: 'Constitutional Research Corpus Analytics Report',
            generated: new Date().toISOString(),
            overview: this.loader.getOverview(),
            top_clauses: this.loader.getTopClauses().slice(0, 20),
            top_issues: this.loader.getTopIssues().slice(0, 15),
            collections: this.loader.getCollections(),
            authors: this.loader.getAuthors().slice(0, 10),
            documents: this.loader.getDocuments(),
            word_frequency: this.loader.getWordFrequency().slice(0, 50),
            chunk_sizes: this.loader.getChunkSizes()
        };

        const json = JSON.stringify(report, null, 2);
        const blob = new Blob([json], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `constitutional-analytics-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        this.showMessage('JSON report downloaded successfully!');
    }

    downloadHTMLReport() {
        const overview = this.loader.getOverview();
        const clauses = this.loader.getTopClauses().slice(0, 20);
        const issues = this.loader.getTopIssues().slice(0, 15);
        const collections = this.loader.getCollections();
        const authors = this.loader.getAuthors().slice(0, 10);

        const html = `<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Constitutional Analytics Report</title>
    <style>
        body { font-family: Arial, sans-serif; line-height: 1.6; max-width: 1000px; margin: 0 auto; padding: 20px; }
        h1 { color: #1a5f7a; border-bottom: 2px solid #1a5f7a; padding-bottom: 10px; }
        h2 { color: #1a5f7a; margin-top: 30px; }
        .stat-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin: 20px 0; }
        .stat-card { background: #f5f7fa; padding: 20px; border-left: 4px solid #1a5f7a; border-radius: 4px; }
        .stat-value { font-size: 2em; font-weight: bold; color: #1a5f7a; }
        .stat-label { font-size: 0.9em; color: #666; text-transform: uppercase; }
        table { width: 100%; border-collapse: collapse; margin: 20px 0; }
        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }
        th { background-color: #1a5f7a; color: white; }
        tr:nth-child(even) { background-color: #f5f7fa; }
        .generated { color: #666; font-size: 0.9em; margin-top: 20px; }
        @media print { body { max-width: 100%; } }
    </style>
</head>
<body>
    <h1>Constitutional Research Corpus Analytics Report</h1>
    <p class="generated">Generated: ${new Date().toLocaleString()}</p>

    <h2>Executive Summary</h2>
    <div class="stat-grid">
        <div class="stat-card">
            <div class="stat-label">Total Chunks</div>
            <div class="stat-value">${overview.total_chunks || 0}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Documents</div>
            <div class="stat-value">${overview.total_documents || 0}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Total Words</div>
            <div class="stat-value">${(overview.total_words || 0).toLocaleString()}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Collections</div>
            <div class="stat-value">${overview.total_collections || 0}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Clauses Referenced</div>
            <div class="stat-value">${overview.total_clause_tags || 0}</div>
        </div>
        <div class="stat-card">
            <div class="stat-label">Issues Discussed</div>
            <div class="stat-value">${overview.total_issue_tags || 0}</div>
        </div>
    </div>

    <h2>Top Constitutional Clauses</h2>
    <table>
        <tr><th>Clause</th><th>Mentions</th><th>Documents</th></tr>
        ${clauses.map(c => `<tr><td>${c.clause}</td><td>${c.count}</td><td>${c.documents}</td></tr>`).join('')}
    </table>

    <h2>Top Thematic Issues</h2>
    <table>
        <tr><th>Issue</th><th>Mentions</th><th>Documents</th></tr>
        ${issues.map(i => `<tr><td>${i.issue.replace(/_/g, ' ')}</td><td>${i.count}</td><td>${i.documents}</td></tr>`).join('')}
    </table>

    <h2>Collections</h2>
    <table>
        <tr><th>Collection</th><th>Chunks</th><th>Documents</th><th>Total Words</th></tr>
        ${collections.map(c => `<tr><td>${c.collection.replace(/_/g, ' ')}</td><td>${c.chunks}</td><td>${c.documents}</td><td>${c.total_words}</td></tr>`).join('')}
    </table>

    <h2>Top Authors</h2>
    <table>
        <tr><th>Author</th><th>Chunks</th><th>Documents</th><th>Unique Clauses</th></tr>
        ${authors.map(a => `<tr><td>${a.author}</td><td>${a.chunks}</td><td>${a.documents}</td><td>${a.unique_clauses}</td></tr>`).join('')}
    </table>
</body>
</html>`;

        const blob = new Blob([html], { type: 'text/html' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `constitutional-analytics-${new Date().toISOString().split('T')[0]}.html`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        this.showMessage('HTML report downloaded successfully!');
    }

    printReport() {
        window.print();
        this.showMessage('Print dialog opened!');
    }
}

// Initialize dashboard when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    new Dashboard();
});
