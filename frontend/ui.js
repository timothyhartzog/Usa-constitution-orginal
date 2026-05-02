/**
 * Constitutional Research System - User Interface
 *
 * Handles all DOM interactions and rendering.
 */

class UI {
    constructor(engine) {
        this.engine = engine;
        this.currentChunkId = null;
        this.selectedChunks = new Set();

        this.initElements();
        this.attachEventListeners();
        this.renderFilters();
        this.showInitialMessage();
    }

    /**
     * Cache DOM elements.
     */
    initElements() {
        this.searchInput = document.getElementById('search-input');
        this.searchBtn = document.getElementById('search-btn');
        this.searchInfo = document.getElementById('search-info');
        this.collectionFilters = document.getElementById('collection-filters');
        this.clauseFilters = document.getElementById('clause-filters');
        this.issueFilters = document.getElementById('issue-filters');
        this.clearFiltersBtn = document.getElementById('clear-filters-btn');
        this.resultsList = document.getElementById('results-list');
        this.resultsCount = document.getElementById('results-count');
        this.passageViewer = document.getElementById('passage-viewer');
        this.passageTitle = document.getElementById('passage-title');
        this.passageText = document.getElementById('passage-text');
        this.passageAuthor = document.getElementById('passage-author');
        this.passageDate = document.getElementById('passage-date');
        this.passageSource = document.getElementById('passage-source');
        this.passageClauses = document.getElementById('passage-clauses');
        this.passageIssues = document.getElementById('passage-issues');
        this.passageSourceLink = document.getElementById('passage-source-link');
        this.passageCloseBtn = document.querySelector('.passage-close-btn');
        this.prevChunkBtn = document.getElementById('prev-chunk-btn');
        this.nextChunkBtn = document.getElementById('next-chunk-btn');
        this.exportChunkBtn = document.getElementById('export-chunk-btn');
        this.loadingIndicator = document.getElementById('loading-indicator');
        this.errorMessage = document.getElementById('error-message');
    }

    /**
     * Attach event listeners.
     */
    attachEventListeners() {
        this.searchBtn.addEventListener('click', () => this.performSearch());
        this.searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') this.performSearch();
        });

        this.clearFiltersBtn.addEventListener('click', () => this.clearFilters());
        this.passageCloseBtn.addEventListener('click', () => this.closePassageViewer());
        this.prevChunkBtn.addEventListener('click', () => this.showPreviousChunk());
        this.nextChunkBtn.addEventListener('click', () => this.showNextChunk());
        this.exportChunkBtn.addEventListener('click', () => this.exportCurrentChunk());

        // Close modal on background click
        this.passageViewer.addEventListener('click', (e) => {
            if (e.target === this.passageViewer) {
                this.closePassageViewer();
            }
        });
    }

    /**
     * Render filter options.
     */
    renderFilters() {
        // Collections
        const collections = this.engine.getCollections();
        this.collectionFilters.innerHTML = '';
        for (const collection of collections) {
            this.collectionFilters.appendChild(
                this.createFilterCheckbox(
                    collection,
                    collection.replace(/_/g, ' ').toUpperCase(),
                    'collections'
                )
            );
        }

        // Clauses
        const clauses = this.engine.getClauses();
        this.clauseFilters.innerHTML = '';
        for (const clause of clauses) {
            const displayName = this.engine.getClauseDisplayName(clause);
            this.clauseFilters.appendChild(
                this.createFilterCheckbox(clause, displayName, 'clauses')
            );
        }

        // Issues
        const issues = this.engine.getIssues();
        this.issueFilters.innerHTML = '';
        for (const issue of issues) {
            const displayName = this.engine.getIssueDisplayName(issue);
            this.issueFilters.appendChild(
                this.createFilterCheckbox(issue, displayName, 'issues')
            );
        }
    }

    /**
     * Create a filter checkbox.
     */
    createFilterCheckbox(value, label, filterType) {
        const label_elem = document.createElement('label');
        label_elem.className = 'filter-checkbox';

        const input = document.createElement('input');
        input.type = 'checkbox';
        input.value = value;

        input.addEventListener('change', () => {
            if (input.checked) {
                this.engine.currentFilters[filterType].push(value);
            } else {
                this.engine.currentFilters[filterType] =
                    this.engine.currentFilters[filterType].filter(v => v !== value);
            }
            this.performSearch();
        });

        label_elem.appendChild(input);
        label_elem.appendChild(document.createTextNode(label));

        return label_elem;
    }

    /**
     * Perform search.
     */
    performSearch() {
        const query = this.searchInput.value.trim();

        if (!query) {
            this.showInitialMessage();
            return;
        }

        this.showLoading();

        // Use setTimeout to allow UI to update
        setTimeout(() => {
            const results = this.engine.search(query, this.engine.currentFilters);
            this.renderResults(results, query);
            this.hideLoading();
        }, 10);
    }

    /**
     * Render search results.
     */
    renderResults(results, query) {
        this.resultsList.innerHTML = '';

        if (results.length === 0) {
            this.resultsList.innerHTML = '<p class="no-results">No results found. Try different keywords.</p>';
            this.resultsCount.textContent = 'No results';
            return;
        }

        this.resultsCount.textContent = `${results.length} result${results.length !== 1 ? 's' : ''}`;

        for (const result of results) {
            const chunk = result.chunk;
            const resultItem = document.createElement('div');
            resultItem.className = 'result-item';

            // Snippet
            let snippet = chunk.text.substring(0, 150);
            if (chunk.text.length > 150) snippet += '...';

            // Highlight query terms
            const tokens = query.split(/\s+/);
            for (const token of tokens) {
                const regex = new RegExp(`\\b(${token})\\b`, 'gi');
                snippet = snippet.replace(regex, '<strong>$1</strong>');
            }

            resultItem.innerHTML = `
                <div class="result-title">${this.escapeHtml(chunk.title)}</div>
                <div class="result-meta">
                    <span>${this.escapeHtml(chunk.author)}</span>
                    <span>${chunk.date}</span>
                    <span>${chunk.word_count} words</span>
                </div>
                <div class="result-snippet">${snippet}</div>
                <div class="result-tags">
                    ${chunk.constitutional_clause_tags.map(tag =>
                        `<span class="tag">${this.engine.getClauseDisplayName(tag)}</span>`
                    ).join('')}
                    ${chunk.issue_tags.map(tag =>
                        `<span class="tag issue">${this.engine.getIssueDisplayName(tag)}</span>`
                    ).join('')}
                </div>
            `;

            resultItem.addEventListener('click', () => this.showPassage(chunk.chunk_id));
            this.resultsList.appendChild(resultItem);
        }
    }

    /**
     * Show initial message when no search performed.
     */
    showInitialMessage() {
        this.resultsList.innerHTML = `
            <div style="text-align: center; padding: 40px; color: #999;">
                <h3>Welcome to the Constitutional Research System</h3>
                <p>Search through founding documents from 1786-1791</p>
                <p style="font-size: 0.9rem; margin-top: 20px;">
                    Try searching for terms like "federalism", "commerce", "constitution",
                    "liberty", or "states"
                </p>
            </div>
        `;
        this.resultsCount.textContent = '';
    }

    /**
     * Show passage viewer modal.
     */
    showPassage(chunkId) {
        const chunk = this.engine.getChunkById(chunkId);
        if (!chunk) return;

        this.currentChunkId = chunkId;

        // Update passage content
        this.passageTitle.textContent = chunk.title;
        this.passageText.textContent = chunk.text;
        this.passageAuthor.innerHTML = `<strong>Author:</strong> ${this.escapeHtml(chunk.author)}`;
        this.passageDate.innerHTML = `<strong>Date:</strong> ${chunk.date}`;
        this.passageSource.innerHTML =
            `<strong>Source:</strong> ${this.escapeHtml(chunk.collection.replace(/_/g, ' '))}`;

        // Update tags
        this.passageClauses.innerHTML = chunk.constitutional_clause_tags.length > 0 ?
            `<h4>Constitutional References:</h4><div class="result-tags">${
                chunk.constitutional_clause_tags.map(tag =>
                    `<span class="tag">${this.engine.getClauseDisplayName(tag)}</span>`
                ).join('')
            }</div>` : '';

        this.passageIssues.innerHTML = chunk.issue_tags.length > 0 ?
            `<h4>Thematic Issues:</h4><div class="result-tags">${
                chunk.issue_tags.map(tag =>
                    `<span class="tag issue">${this.engine.getIssueDisplayName(tag)}</span>`
                ).join('')
            }</div>` : '';

        // Update source link
        this.passageSourceLink.href = chunk.source_url;

        // Update navigation buttons
        const prevChunk = this.engine.getPreviousChunk(chunkId);
        const nextChunk = this.engine.getNextChunk(chunkId);

        this.prevChunkBtn.disabled = !prevChunk;
        this.nextChunkBtn.disabled = !nextChunk;

        // Show modal
        this.passageViewer.classList.remove('hidden');
        this.passageViewer.scrollTop = 0;
    }

    /**
     * Show previous chunk in document.
     */
    showPreviousChunk() {
        if (!this.currentChunkId) return;
        const prevChunk = this.engine.getPreviousChunk(this.currentChunkId);
        if (prevChunk) {
            this.showPassage(prevChunk.chunk_id);
        }
    }

    /**
     * Show next chunk in document.
     */
    showNextChunk() {
        if (!this.currentChunkId) return;
        const nextChunk = this.engine.getNextChunk(this.currentChunkId);
        if (nextChunk) {
            this.showPassage(nextChunk.chunk_id);
        }
    }

    /**
     * Close passage viewer.
     */
    closePassageViewer() {
        this.passageViewer.classList.add('hidden');
        this.currentChunkId = null;
    }

    /**
     * Export current chunk.
     */
    exportCurrentChunk() {
        if (!this.currentChunkId) return;

        const chunk = this.engine.getChunkById(this.currentChunkId);
        if (!chunk) return;

        // Create export data
        const exportData = {
            chunks: [chunk],
            exported_at: new Date().toISOString(),
            source_url: chunk.source_url
        };

        // Offer download
        const json = JSON.stringify(exportData, null, 2);
        const blob = new Blob([json], { type: 'application/json' });
        const url = URL.createObjectURL(blob);

        const a = document.createElement('a');
        a.href = url;
        a.download = `${chunk.chunk_id}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        this.showMessage('Chunk exported successfully!');
    }

    /**
     * Clear all filters.
     */
    clearFilters() {
        this.engine.currentFilters = {
            collections: [],
            clauses: [],
            issues: []
        };

        // Uncheck all checkboxes
        document.querySelectorAll('.filter-checkbox input').forEach(input => {
            input.checked = false;
        });

        this.performSearch();
    }

    /**
     * Show loading indicator.
     */
    showLoading() {
        this.loadingIndicator.classList.remove('hidden');
    }

    /**
     * Hide loading indicator.
     */
    hideLoading() {
        this.loadingIndicator.classList.add('hidden');
    }

    /**
     * Show error message.
     */
    showError(message) {
        this.errorMessage.textContent = message;
        this.errorMessage.classList.remove('hidden');
        setTimeout(() => this.errorMessage.classList.add('hidden'), 5000);
    }

    /**
     * Show temporary message.
     */
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

        setTimeout(() => {
            msg.remove();
        }, 3000);
    }

    /**
     * Escape HTML special characters.
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize UI when search engine is ready
document.addEventListener('searchEngineReady', (event) => {
    const engine = event.detail;
    new UI(engine);
});

// Handle search engine initialization errors
document.addEventListener('searchEngineError', (event) => {
    const message = event.detail;
    const errorDiv = document.createElement('div');
    errorDiv.style.cssText = `
        padding: 20px;
        background: #f8d7da;
        color: #721c24;
        border: 1px solid #f5c6cb;
        border-radius: 4px;
        margin: 20px;
    `;
    errorDiv.innerHTML = `
        <h3>Error Loading Research System</h3>
        <p>${message}</p>
        <p style="font-size: 0.9rem; margin-top: 10px;">
            Please ensure all data files have been generated.<br>
            Run: python scripts/ingest_sources.py && python scripts/clean_text.py &&
            python scripts/chunk_documents.py && python scripts/build_search_index.py
        </p>
    `;
    document.body.appendChild(errorDiv);
});
