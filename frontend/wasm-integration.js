/**
 * WASM Integration Layer
 * Provides high-level API for search and analytics using Rust WASM modules
 */

class WasmSearchEngine {
    constructor() {
        this.engine = null;
        this.loaded = false;
        this.wasmModule = null;
    }

    async init() {
        if (this.loaded) return true;

        try {
            // Load WASM module
            const wasmModule = await import('./wasm/constitutional_wasm.js');
            this.wasmModule = wasmModule;

            // Initialize search engine
            this.engine = new wasmModule.WasmSearchEngine();
            console.log('✅ WASM Search Engine initialized');
            this.loaded = true;
            return true;
        } catch (error) {
            console.error('❌ Failed to initialize WASM:', error);
            return false;
        }
    }

    async loadCorpus(corpusJson) {
        if (!this.engine) {
            throw new Error('WASM not initialized');
        }

        try {
            const result = this.engine.load_corpus(corpusJson);
            console.log('✅ Corpus loaded:', result);
            return result;
        } catch (error) {
            console.error('❌ Failed to load corpus:', error);
            throw error;
        }
    }

    search(query, limit = 50) {
        if (!this.engine) {
            throw new Error('WASM not initialized');
        }

        try {
            const results = this.engine.search(query, limit);
            return JSON.parse(results);
        } catch (error) {
            console.error('❌ Search failed:', error);
            return [];
        }
    }

    searchWithFilters(query, filters, limit = 50) {
        if (!this.engine) {
            throw new Error('WASM not initialized');
        }

        try {
            const filtersJson = JSON.stringify(filters);
            const results = this.engine.search_with_filters(query, filtersJson, limit);
            return JSON.parse(results);
        } catch (error) {
            console.error('❌ Search with filters failed:', error);
            return [];
        }
    }

    getChunk(chunkId) {
        if (!this.engine) {
            throw new Error('WASM not initialized');
        }

        try {
            const chunk = this.engine.get_chunk(chunkId);
            return JSON.parse(chunk);
        } catch (error) {
            console.error('❌ Failed to get chunk:', error);
            return null;
        }
    }

    getFilters() {
        if (!this.engine) {
            throw new Error('WASM not initialized');
        }

        try {
            const filters = this.engine.get_filters();
            return JSON.parse(filters);
        } catch (error) {
            console.error('❌ Failed to get filters:', error);
            return {};
        }
    }

    getStats() {
        if (!this.engine) {
            throw new Error('WASM not initialized');
        }

        try {
            const stats = this.engine.get_stats();
            return JSON.parse(stats);
        } catch (error) {
            console.error('❌ Failed to get stats:', error);
            return {};
        }
    }
}

class WasmAnalyticsEngine {
    constructor() {
        this.analytics = null;
        this.loaded = false;
        this.wasmModule = null;
    }

    async init() {
        if (this.loaded) return true;

        try {
            // Load WASM module (same as search)
            const wasmModule = await import('./wasm/constitutional_wasm.js');
            this.wasmModule = wasmModule;

            // Initialize analytics engine
            this.analytics = new wasmModule.WasmAnalytics();
            console.log('✅ WASM Analytics Engine initialized');
            this.loaded = true;
            return true;
        } catch (error) {
            console.error('❌ Failed to initialize WASM Analytics:', error);
            return false;
        }
    }

    async loadCorpus(corpusJson) {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const result = this.analytics.load_corpus(corpusJson);
            console.log('✅ Corpus loaded for analytics:', result);
            return result;
        } catch (error) {
            console.error('❌ Failed to load corpus for analytics:', error);
            throw error;
        }
    }

    analyzeOverview() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const overview = this.analytics.analyze_overview();
            return JSON.parse(overview);
        } catch (error) {
            console.error('❌ Failed to analyze overview:', error);
            return {};
        }
    }

    analyzeClauses(topN = 30) {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const clauses = this.analytics.analyze_clauses(topN);
            return JSON.parse(clauses);
        } catch (error) {
            console.error('❌ Failed to analyze clauses:', error);
            return [];
        }
    }

    analyzeIssues(topN = 20) {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const issues = this.analytics.analyze_issues(topN);
            return JSON.parse(issues);
        } catch (error) {
            console.error('❌ Failed to analyze issues:', error);
            return [];
        }
    }

    analyzeAuthors() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const authors = this.analytics.analyze_authors();
            return JSON.parse(authors);
        } catch (error) {
            console.error('❌ Failed to analyze authors:', error);
            return [];
        }
    }

    analyzeCollections() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const collections = this.analytics.analyze_collections();
            return JSON.parse(collections);
        } catch (error) {
            console.error('❌ Failed to analyze collections:', error);
            return [];
        }
    }

    analyzeWordFrequency(topN = 50) {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const words = this.analytics.analyze_word_frequency(topN);
            return JSON.parse(words);
        } catch (error) {
            console.error('❌ Failed to analyze word frequency:', error);
            return [];
        }
    }

    analyzeClauseIssueMatrix() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const matrix = this.analytics.analyze_clause_issue_matrix();
            return JSON.parse(matrix);
        } catch (error) {
            console.error('❌ Failed to analyze clause-issue matrix:', error);
            return {};
        }
    }

    analyzeAuthorClauseMatrix() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const matrix = this.analytics.analyze_author_clause_matrix();
            return JSON.parse(matrix);
        } catch (error) {
            console.error('❌ Failed to analyze author-clause matrix:', error);
            return {};
        }
    }

    filterChunks(filters) {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const filtersJson = JSON.stringify(filters);
            const chunks = this.analytics.filter_chunks(filtersJson);
            return JSON.parse(chunks);
        } catch (error) {
            console.error('❌ Failed to filter chunks:', error);
            return [];
        }
    }

    compareAuthors(author1, author2) {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const comparison = this.analytics.compare_authors(author1, author2);
            return JSON.parse(comparison);
        } catch (error) {
            console.error('❌ Failed to compare authors:', error);
            return {};
        }
    }

    analyzeTemporalNetwork() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const network = this.analytics.analyze_temporal_network();
            return JSON.parse(network);
        } catch (error) {
            console.error('❌ Failed to analyze temporal network:', error);
            return {};
        }
    }

    analyzeClauseDebateNetwork() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const network = this.analytics.analyze_clause_debate_network();
            return JSON.parse(network);
        } catch (error) {
            console.error('❌ Failed to analyze clause debate network:', error);
            return {};
        }
    }

    analyzeAuthorInfluence() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const influence = this.analytics.analyze_author_influence();
            return JSON.parse(influence);
        } catch (error) {
            console.error('❌ Failed to analyze author influence:', error);
            return [];
        }
    }

    analyzeSemanticSimilarity() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const clusters = this.analytics.analyze_semantic_similarity();
            return JSON.parse(clusters);
        } catch (error) {
            console.error('❌ Failed to analyze semantic similarity:', error);
            return {};
        }
    }

    analyzeRatification() {
        if (!this.analytics) {
            throw new Error('WASM Analytics not initialized');
        }

        try {
            const ratification = this.analytics.analyze_ratification();
            return JSON.parse(ratification);
        } catch (error) {
            console.error('❌ Failed to analyze ratification:', error);
            return {};
        }
    }
}

// Global instances
window.wasmSearch = new WasmSearchEngine();
window.wasmAnalytics = new WasmAnalyticsEngine();

console.log('✅ WASM Integration Layer loaded');
console.log('Available: window.wasmSearch, window.wasmAnalytics');
