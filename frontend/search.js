/**
 * Constitutional Research System - Client-Side Search Engine
 *
 * Pure JavaScript implementation with no external dependencies.
 * Loads and indexes the full corpus for fast searches.
 */

class ConstitutionalSearchEngine {
    constructor() {
        this.corpus = null;
        this.searchIndex = null;
        this.currentResults = [];
        this.currentFilters = {
            collections: [],
            clauses: [],
            issues: []
        };
    }

    /**
     * Initialize the search engine by loading data files.
     */
    async init() {
        try {
            console.log('Initializing Constitutional Search Engine...');

            // Load corpus
            const corpusResponse = await fetch('../data/chunks/constitution_full_corpus.json');
            if (!corpusResponse.ok) throw new Error('Failed to load corpus');
            this.corpus = await corpusResponse.json();
            console.log(`Loaded ${this.corpus.chunks.length} chunks`);

            // Load search index
            const indexResponse = await fetch('../data/index/search_index.json');
            if (!indexResponse.ok) throw new Error('Failed to load search index');
            this.searchIndex = await indexResponse.json();
            console.log('Search index loaded');

            return true;
        } catch (error) {
            console.error('Failed to initialize search engine:', error);
            throw error;
        }
    }

    /**
     * Tokenize a search query.
     */
    tokenizeQuery(query) {
        return query.toLowerCase()
            .replace(/[^\w\s-]/g, '')
            .split(/\s+/)
            .filter(word => word.length > 2);
    }

    /**
     * Search corpus with optional filters.
     */
    search(query, filters = {}) {
        const tokens = this.tokenizeQuery(query);

        if (tokens.length === 0) {
            this.currentResults = [];
            return this.currentResults;
        }

        let results = [];

        // Find chunks containing all query tokens
        const chunkScores = {};

        for (const token of tokens) {
            if (this.searchIndex.words[token]) {
                for (const chunkId of this.searchIndex.words[token]) {
                    chunkScores[chunkId] = (chunkScores[chunkId] || 0) + 1;
                }
            }
        }

        // Get chunks matching all tokens (AND logic)
        for (const [chunkId, score] of Object.entries(chunkScores)) {
            if (score === tokens.length) {
                const chunk = this.getChunkById(chunkId);
                if (chunk) {
                    results.push({
                        chunk,
                        score: score,
                        relevance: this.calculateRelevance(chunk, tokens)
                    });
                }
            }
        }

        // Apply filters
        if (filters.collections && filters.collections.length > 0) {
            results = results.filter(r =>
                filters.collections.includes(r.chunk.collection)
            );
        }

        if (filters.clauses && filters.clauses.length > 0) {
            results = results.filter(r => {
                const chunkClauses = r.chunk.constitutional_clause_tags || [];
                return filters.clauses.some(c => chunkClauses.includes(c));
            });
        }

        if (filters.issues && filters.issues.length > 0) {
            results = results.filter(r => {
                const chunkIssues = r.chunk.issue_tags || [];
                return filters.issues.some(i => chunkIssues.includes(i));
            });
        }

        // Sort by relevance
        results.sort((a, b) => b.relevance - a.relevance);

        this.currentResults = results;
        return results;
    }

    /**
     * Calculate relevance score for a chunk.
     */
    calculateRelevance(chunk, tokens) {
        const text = chunk.text.toLowerCase();
        let score = 0;

        // Token frequency
        for (const token of tokens) {
            const regex = new RegExp(`\\b${token}\\b`, 'g');
            const matches = text.match(regex);
            if (matches) {
                score += matches.length;
            }
        }

        // Boost if tokens appear near beginning
        const beginning = text.substring(0, 100);
        for (const token of tokens) {
            if (beginning.includes(token)) {
                score += 2;
            }
        }

        return score;
    }

    /**
     * Get chunk by ID.
     */
    getChunkById(chunkId) {
        return this.corpus.chunks.find(c => c.chunk_id === chunkId);
    }

    /**
     * Get chunk by index.
     */
    getChunkByIndex(index) {
        return this.corpus.chunks[index];
    }

    /**
     * Find index of chunk in corpus.
     */
    getChunkIndex(chunkId) {
        return this.corpus.chunks.findIndex(c => c.chunk_id === chunkId);
    }

    /**
     * Get next chunk in document.
     */
    getNextChunk(chunkId) {
        const index = this.getChunkIndex(chunkId);
        if (index >= 0 && index < this.corpus.chunks.length - 1) {
            return this.corpus.chunks[index + 1];
        }
        return null;
    }

    /**
     * Get previous chunk in document.
     */
    getPreviousChunk(chunkId) {
        const index = this.getChunkIndex(chunkId);
        if (index > 0) {
            return this.corpus.chunks[index - 1];
        }
        return null;
    }

    /**
     * Get all unique collections.
     */
    getCollections() {
        return this.searchIndex.filters.collections;
    }

    /**
     * Get all unique constitutional clauses.
     */
    getClauses() {
        return this.searchIndex.filters.clauses || [];
    }

    /**
     * Get all unique issue tags.
     */
    getIssues() {
        return this.searchIndex.filters.issues || [];
    }

    /**
     * Get clause display name.
     */
    getClauseDisplayName(clauseId) {
        // Map clause_id to display name
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

    /**
     * Get issue display name.
     */
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

    /**
     * Export chunks to JSON format.
     */
    exportChunksToJSON(chunks) {
        return JSON.stringify(chunks, null, 2);
    }

    /**
     * Export chunks to CSV format.
     */
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

/**
 * Initialize the application.
 */
async function initializeApp() {
    try {
        searchEngine = new ConstitutionalSearchEngine();
        await searchEngine.init();

        // Notify UI that engine is ready
        document.dispatchEvent(new CustomEvent('searchEngineReady', { detail: searchEngine }));

    } catch (error) {
        console.error('Failed to initialize app:', error);
        document.dispatchEvent(new CustomEvent('searchEngineError', { detail: error.message }));
    }
}

// Initialize on page load
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initializeApp);
} else {
    initializeApp();
}
