/**
 * Hybrid Search Engine - WASM + JavaScript Fallback
 *
 * Automatically uses WASM for superior performance when available,
 * falls back to pure JavaScript for compatibility.
 */

class HybridSearchEngine {
    constructor() {
        this.useWasm = false;
        this.wasmEngine = null;
        this.jsEngine = null;
        this.corpus = null;
        this.currentResults = [];
        this.currentFilters = {
            collections: [],
            clauses: [],
            issues: []
        };
    }

    async init() {
        try {
            // Load corpus first (needed by both engines)
            console.log('Loading corpus...');
            const corpusResponse = await fetch('../data/chunks/constitution_full_corpus.json');
            if (!corpusResponse.ok) throw new Error('Failed to load corpus');
            this.corpus = await corpusResponse.json();
            console.log(`✅ Loaded ${this.corpus.chunks.length} chunks`);

            // Try to initialize WASM
            const wasmLoaded = await this.initWasm();

            if (wasmLoaded) {
                console.log('🚀 Using WASM-accelerated search (Rust)');
                this.useWasm = true;
            } else {
                console.log('⚠️  WASM not available, falling back to JavaScript');
                this.useWasm = false;
                await this.initJavaScript();
            }

            return true;
        } catch (error) {
            console.error('❌ Failed to initialize search engine:', error);
            throw error;
        }
    }

    async initWasm() {
        try {
            // Initialize WASM search engine
            await window.wasmSearch.init();

            // Load corpus JSON as string
            const corpusJson = JSON.stringify(this.corpus);
            await window.wasmSearch.loadCorpus(corpusJson);

            this.wasmEngine = window.wasmSearch;
            return true;
        } catch (error) {
            console.warn('⚠️  WASM initialization failed:', error);
            return false;
        }
    }

    async initJavaScript() {
        try {
            // Initialize JavaScript fallback
            this.jsEngine = new ConstitutionalSearchEngine();

            // Manually set corpus and load index
            this.jsEngine.corpus = this.corpus;
            const indexResponse = await fetch('../data/index/search_index.json');
            if (!indexResponse.ok) throw new Error('Failed to load search index');
            this.jsEngine.searchIndex = await indexResponse.json();

            console.log('✅ JavaScript search engine ready');
            return true;
        } catch (error) {
            console.error('❌ JavaScript fallback failed:', error);
            throw error;
        }
    }

    search(query, filters = {}) {
        try {
            let results;

            if (this.useWasm && this.wasmEngine) {
                // Use WASM for superior performance
                results = this.wasmEngine.search(query, 50);

                // Convert WASM results to our format
                return this.formatWasmResults(results, filters);
            } else if (this.jsEngine) {
                // Fallback to JavaScript
                const jsResults = this.jsEngine.search(query, filters);
                return this.formatJsResults(jsResults);
            }

            return [];
        } catch (error) {
            console.error('Search failed:', error);
            return [];
        }
    }

    formatWasmResults(wasmResults, filters) {
        // Apply filters if needed
        let filtered = wasmResults;

        if (filters.collections && filters.collections.length > 0) {
            filtered = filtered.filter(r => {
                const collection = r.chunk_id.split('_')[0];
                return filters.collections.includes(collection);
            });
        }

        if (filters.clauses && filters.clauses.length > 0) {
            filtered = filtered.filter(r => {
                const chunk = this.corpus.chunks.find(c => c.chunk_id === r.chunk_id);
                if (!chunk) return false;
                return filters.clauses.some(c => chunk.constitutional_clause_tags.includes(c));
            });
        }

        if (filters.issues && filters.issues.length > 0) {
            filtered = filtered.filter(r => {
                const chunk = this.corpus.chunks.find(c => c.chunk_id === r.chunk_id);
                if (!chunk) return false;
                return filters.issues.some(i => chunk.issue_tags.includes(i));
            });
        }

        // Format for UI
        const formatted = filtered.map(result => {
            const chunk = this.corpus.chunks.find(c => c.chunk_id === result.chunk_id);
            return {
                chunk: chunk,
                score: result.relevance_score,
                relevance: result.relevance_score
            };
        });

        this.currentResults = formatted;
        return formatted;
    }

    formatJsResults(jsResults) {
        this.currentResults = jsResults;
        return jsResults;
    }

    getChunkById(chunkId) {
        return this.corpus.chunks.find(c => c.chunk_id === chunkId);
    }

    getChunkByIndex(index) {
        return this.corpus.chunks[index];
    }

    getChunkIndex(chunkId) {
        return this.corpus.chunks.findIndex(c => c.chunk_id === chunkId);
    }

    getNextChunk(chunkId) {
        const index = this.getChunkIndex(chunkId);
        if (index >= 0 && index < this.corpus.chunks.length - 1) {
            return this.corpus.chunks[index + 1];
        }
        return null;
    }

    getPreviousChunk(chunkId) {
        const index = this.getChunkIndex(chunkId);
        if (index > 0) {
            return this.corpus.chunks[index - 1];
        }
        return null;
    }

    getCollections() {
        if (this.useWasm && this.wasmEngine) {
            const filters = this.wasmEngine.getFilters();
            return filters.collections || [];
        } else if (this.jsEngine) {
            return this.jsEngine.getCollections();
        }
        return [];
    }

    getClauses() {
        if (this.useWasm && this.wasmEngine) {
            const filters = this.wasmEngine.getFilters();
            return filters.clauses || [];
        } else if (this.jsEngine) {
            return this.jsEngine.getClauses();
        }
        return [];
    }

    getIssues() {
        if (this.useWasm && this.wasmEngine) {
            const filters = this.wasmEngine.getFilters();
            return filters.issues || [];
        } else if (this.jsEngine) {
            return this.jsEngine.getIssues();
        }
        return [];
    }

    getClauseDisplayName(clauseId) {
        const clauseNames = {
            'preamble': 'Preamble',
            'I.1': 'Legislative Power',
            'I.2': 'House of Representatives',
            'I.3': 'Senate',
            'I.4': 'Elections and Meetings',
            'I.5': 'Proceedings',
            'I.6': 'Compensation',
            'I.7': 'Legislative Process',
            'I.8': 'Enumerated Powers',
            'I.9': 'Limitations on Congress',
            'I.10': 'Limitations on States',
            'II.1': 'Executive Power',
            'II.2': 'Commander in Chief',
            'II.3': 'State of the Union',
            'II.4': 'Take Care Clause',
            'III.1': 'Judicial Power',
            'III.2': 'Scope of Judicial Power',
            'III.3': 'Treason',
            'IV.1': 'Full Faith and Credit',
            'IV.2': 'Privileges and Immunities',
            'IV.3': 'New States',
            'IV.4': 'Guarantee Clause',
            'V': 'Amendment Process',
            'VI.1': 'Supremacy Clause',
            'VI.2': 'Oath Clause',
            'VII': 'Ratification'
        };
        return clauseNames[clauseId] || clauseId;
    }

    getIssueDisplayName(issueId) {
        const issueNames = {
            'federalism': 'Federalism',
            'separation_of_powers': 'Separation of Powers',
            'representation': 'Representation',
            'commerce': 'Commerce',
            'rights': 'Rights & Liberties',
            'judicial_review': 'Judicial Review',
            'amending_process': 'Amending Process',
            'ratification': 'Ratification',
            'executive_power': 'Executive Power',
            'political_theory': 'Political Theory'
        };
        return issueNames[issueId] || issueId;
    }

    exportChunksToJSON(chunks) {
        return JSON.stringify(chunks, null, 2);
    }

    exportChunksToCSV(chunks) {
        const headers = [
            'chunk_id',
            'document_id',
            'title',
            'author',
            'date',
            'word_count',
            'source_url',
            'constitutional_clause_tags',
            'issue_tags'
        ];

        let csv = headers.join(',') + '\n';

        for (const chunk of chunks) {
            const row = [
                chunk.chunk_id,
                chunk.document_id,
                `"${chunk.title.replace(/"/g, '""')}"`,
                `"${chunk.author.replace(/"/g, '""')}"`,
                chunk.date,
                chunk.word_count,
                `"${chunk.source_url}"`,
                `"${chunk.constitutional_clause_tags.join('|')}"`,
                `"${chunk.issue_tags.join('|')}"`
            ];
            csv += row.join(',') + '\n';
        }

        return csv;
    }
}

// Create global search engine instance
let searchEngine = null;

async function initializeApp() {
    try {
        searchEngine = new HybridSearchEngine();
        await searchEngine.init();

        // Notify UI that engine is ready
        document.dispatchEvent(new CustomEvent('searchEngineReady', { detail: searchEngine }));

    } catch (error) {
        console.error('❌ Failed to initialize app:', error);
        document.dispatchEvent(new CustomEvent('searchEngineError', { detail: error.message }));
    }
}

// Initialize on page load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    initializeApp();
}
