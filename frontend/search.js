class ConstitutionalSearchEngine {
    constructor() {
        this.corpus = null;
        this.index = null;
        this.chunkMap = new Map();
    }

    async init() {
        const [corpusResponse, indexResponse] = await Promise.all([
            fetch('../data/chunks/constitution_full_corpus.json'),
            fetch('../data/index/search_index.json'),
        ]);

        if (!corpusResponse.ok || !indexResponse.ok) {
            throw new Error('Failed to load corpus data. Run the Python build scripts first.');
        }

        this.corpus = await corpusResponse.json();
        this.index = await indexResponse.json();
        this.chunkMap = new Map(this.corpus.chunks.map((chunk) => [chunk.chunk_id, chunk]));
        return this;
    }

    tokenizeQuery(query) {
        return query
            .toLowerCase()
            .match(/[a-z0-9']+/g)?.filter((token) => token.length > 2) || [];
    }

    getFilterData() {
        return this.index.filters;
    }

    getAllChunks() {
        return this.corpus.chunks;
    }

    getChunk(chunkId) {
        return this.chunkMap.get(chunkId) || null;
    }

    countMatches(text, token) {
        const matches = text.match(new RegExp(`\\b${token}\\b`, 'gi'));
        return matches ? matches.length : 0;
    }

    search(query, filters = {}) {
        const tokens = this.tokenizeQuery(query);
        let candidateIds;

        if (tokens.length === 0) {
            candidateIds = this.corpus.chunks.map((chunk) => chunk.chunk_id);
        } else {
            candidateIds = tokens
                .map((token) => this.index.inverted_index[token] || [])
                .reduce((accumulator, ids) => {
                    if (accumulator === null) {
                        return [...ids];
                    }
                    return accumulator.filter((id) => ids.includes(id));
                }, null) || [];
        }

        return candidateIds
            .map((chunkId) => this.getChunk(chunkId))
            .filter(Boolean)
            .filter((chunk) => !filters.sourceCollection || chunk.source_collection === filters.sourceCollection)
            .filter((chunk) => !filters.documentId || chunk.document_id === filters.documentId)
            .filter((chunk) => !filters.author || chunk.author === filters.author)
            .filter((chunk) => !filters.issue || chunk.issue_tags.includes(filters.issue))
            .map((chunk) => ({
                chunk,
                score: tokens.reduce((score, token) => {
                    return score
                        + this.countMatches(`${chunk.title} ${chunk.text}`, token)
                        + (chunk.title.toLowerCase().includes(token) ? 3 : 0);
                }, 0),
            }))
            .sort((left, right) => right.score - left.score || left.chunk.title.localeCompare(right.chunk.title));
    }

    exportAsJson(chunks) {
        return JSON.stringify(chunks, null, 2);
    }

    exportAsCsv(chunks) {
        const headers = [
            'chunk_id',
            'document_id',
            'title',
            'author',
            'date',
            'source_collection',
            'source_url',
            'document_type',
            'issue_tags',
            'constitutional_clause_tags',
            'word_count',
            'text',
        ];
        const escapeCell = (value) => `"${String(value).replace(/"/g, '""')}"`;
        const rows = chunks.map((chunk) => [
            chunk.chunk_id,
            chunk.document_id,
            chunk.title,
            chunk.author,
            chunk.date,
            chunk.source_collection,
            chunk.source_url,
            chunk.document_type,
            chunk.issue_tags.join('|'),
            chunk.constitutional_clause_tags.join('|'),
            chunk.word_count,
            chunk.text.replace(/\s+/g, ' ').trim(),
        ]);
        return [headers, ...rows].map((row) => row.map(escapeCell).join(',')).join('\n');
    }
}

window.ConstitutionalSearchEngine = ConstitutionalSearchEngine;
