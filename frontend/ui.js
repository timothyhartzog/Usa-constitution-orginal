(async () => {
    const engine = await new window.ConstitutionalSearchEngine().init();
    const selectedChunkIds = new Set();

    const elements = {
        searchInput: document.querySelector('#search-input'),
        searchButton: document.querySelector('#search-button'),
        collectionFilter: document.querySelector('#collection-filter'),
        documentFilter: document.querySelector('#document-filter'),
        authorFilter: document.querySelector('#author-filter'),
        issueFilter: document.querySelector('#issue-filter'),
        resultsSummary: document.querySelector('#results-summary'),
        resultsList: document.querySelector('#results-list'),
        selectionCount: document.querySelector('#selection-count'),
        selectVisibleButton: document.querySelector('#select-visible-button'),
        clearSelectionButton: document.querySelector('#clear-selection-button'),
        exportJsonButton: document.querySelector('#export-json-button'),
        exportCsvButton: document.querySelector('#export-csv-button'),
        viewerDialog: document.querySelector('#viewer-dialog'),
        viewerClose: document.querySelector('#viewer-close'),
        viewerMeta: document.querySelector('#viewer-meta'),
        viewerTitle: document.querySelector('#viewer-title'),
        viewerTags: document.querySelector('#viewer-tags'),
        viewerText: document.querySelector('#viewer-text'),
        viewerSource: document.querySelector('#viewer-source'),
    };

    let visibleResults = [];

    const download = (filename, content, type) => {
        const blob = new Blob([content], { type });
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = filename;
        link.click();
        URL.revokeObjectURL(url);
    };

    const humanize = (value) => value.replace(/_/g, ' ');

    const currentFilters = () => ({
        sourceCollection: elements.collectionFilter.value,
        documentId: elements.documentFilter.value,
        author: elements.authorFilter.value,
        issue: elements.issueFilter.value,
    });

    const populateSelect = (select, options, formatter = (value) => value) => {
        for (const option of options) {
            const element = document.createElement('option');
            if (typeof option === 'string') {
                element.value = option;
                element.textContent = formatter(option);
            } else {
                element.value = option.document_id;
                element.textContent = `${option.title} — ${option.source_collection}`;
            }
            select.appendChild(element);
        }
    };

    const renderSelectionCount = () => {
        elements.selectionCount.textContent = `${selectedChunkIds.size} selected`;
    };

    const openViewer = (chunk) => {
        elements.viewerMeta.textContent = `${chunk.author} | ${chunk.date} | ${chunk.source_collection}`;
        elements.viewerTitle.textContent = chunk.title;
        elements.viewerText.textContent = chunk.text;
        elements.viewerSource.href = chunk.source_url;
        elements.viewerTags.innerHTML = '';
        [...chunk.issue_tags.map((tag) => ({ label: humanize(tag), className: 'tag tag--issue' })), ...chunk.constitutional_clause_tags.map((tag) => ({ label: tag, className: 'tag' }))].forEach((tag) => {
            const span = document.createElement('span');
            span.className = tag.className;
            span.textContent = tag.label;
            elements.viewerTags.appendChild(span);
        });
        elements.viewerDialog.showModal();
    };

    const renderResults = () => {
        visibleResults = engine.search(elements.searchInput.value, currentFilters());
        elements.resultsSummary.textContent = `${visibleResults.length} matching chunks`;
        elements.resultsList.innerHTML = '';

        if (visibleResults.length === 0) {
            elements.resultsList.innerHTML = '<p class="empty-state">No passages matched the current search and filters.</p>';
            return;
        }

        for (const result of visibleResults) {
            const { chunk } = result;
            const article = document.createElement('article');
            article.className = 'result-card';
            article.innerHTML = `
                <div class="result-card__header">
                    <label class="selection-box">
                        <input type="checkbox" data-select="${chunk.chunk_id}" ${selectedChunkIds.has(chunk.chunk_id) ? 'checked' : ''}>
                        <span>Select</span>
                    </label>
                    <button class="link-button" data-open="${chunk.chunk_id}">Open passage</button>
                </div>
                <h3>${chunk.title}</h3>
                <p class="result-card__meta">${chunk.author} | ${chunk.date} | ${chunk.source_collection} | ${chunk.word_count} words</p>
                <p class="result-card__text">${chunk.preview || chunk.text.slice(0, 240)} ...</p>
                <div class="tag-row">
                    ${chunk.issue_tags.map((tag) => `<span class="tag tag--issue">${humanize(tag)}</span>`).join('')}
                    ${chunk.constitutional_clause_tags.map((tag) => `<span class="tag">${tag}</span>`).join('')}
                </div>
            `;
            elements.resultsList.appendChild(article);
        }

        renderSelectionCount();
    };

    const exportSelection = (format) => {
        const chunks = [...selectedChunkIds]
            .map((chunkId) => engine.getChunk(chunkId))
            .filter(Boolean);
        if (chunks.length === 0) {
            return;
        }
        if (format === 'json') {
            download('selected_chunks.json', engine.exportAsJson(chunks), 'application/json');
        } else {
            download('selected_chunks.csv', engine.exportAsCsv(chunks), 'text/csv');
        }
    };

    populateSelect(elements.collectionFilter, engine.getFilterData().source_collections, humanize);
    populateSelect(elements.documentFilter, engine.getFilterData().documents);
    populateSelect(elements.authorFilter, engine.getFilterData().authors);
    populateSelect(elements.issueFilter, engine.getFilterData().issues, humanize);

    elements.searchButton.addEventListener('click', renderResults);
    elements.searchInput.addEventListener('keydown', (event) => {
        if (event.key === 'Enter') {
            renderResults();
        }
    });
    [elements.collectionFilter, elements.documentFilter, elements.authorFilter, elements.issueFilter].forEach((element) => {
        element.addEventListener('change', renderResults);
    });

    elements.resultsList.addEventListener('click', (event) => {
        const openTarget = event.target.closest('[data-open]');
        const selectTarget = event.target.closest('[data-select]');
        if (openTarget) {
            const chunk = engine.getChunk(openTarget.dataset.open);
            if (chunk) {
                openViewer(chunk);
            }
        }
        if (selectTarget && event.target.type === 'checkbox') {
            if (event.target.checked) {
                selectedChunkIds.add(selectTarget.dataset.select);
            } else {
                selectedChunkIds.delete(selectTarget.dataset.select);
            }
            renderSelectionCount();
        }
    });

    elements.selectVisibleButton.addEventListener('click', () => {
        visibleResults.forEach((result) => selectedChunkIds.add(result.chunk.chunk_id));
        renderResults();
    });

    elements.clearSelectionButton.addEventListener('click', () => {
        selectedChunkIds.clear();
        renderResults();
    });

    elements.exportJsonButton.addEventListener('click', () => exportSelection('json'));
    elements.exportCsvButton.addEventListener('click', () => exportSelection('csv'));
    elements.viewerClose.addEventListener('click', () => elements.viewerDialog.close());
    elements.viewerDialog.addEventListener('click', (event) => {
        if (event.target === elements.viewerDialog) {
            elements.viewerDialog.close();
        }
    });

    renderResults();
})().catch((error) => {
    const summary = document.querySelector('#results-summary');
    if (summary) {
        summary.textContent = error.message;
    }
});
