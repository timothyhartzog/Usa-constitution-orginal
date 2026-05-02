# Frontend API Documentation

## Overview

The Constitutional Research System frontend provides a pure JavaScript client-side search engine with no backend dependencies. All operations execute in the browser using data loaded from JSON files.

## ConstitutionalSearchEngine Class

Main search engine class handling corpus indexing and search operations.

### Initialization

```javascript
// Automatically initialized on page load
// Available globally as: searchEngine

// Initialize manually:
const engine = new ConstitutionalSearchEngine();
await engine.init();
```

### Properties

#### `corpus` (Object)
The loaded chunk corpus with metadata.

**Structure:**
```javascript
{
  metadata: {
    version: "1.0",
    total_chunks: 3247,
    total_documents: 7
  },
  chunks: [...]  // Array of chunk objects
}
```

#### `searchIndex` (Object)
Pre-computed inverted index for fast searches.

**Structure:**
```javascript
{
  metadata: { ... },
  words: {
    "constitution": ["chunk_id_1", "chunk_id_2", ...],
    "power": ["chunk_id_3", ...],
    ...
  },
  clauses: {
    "I.1": ["chunk_id_1", ...],
    "federalism": ["chunk_id_2", ...],
    ...
  },
  filters: {
    collections: ["constitution", "federalist_papers", ...],
    authors: ["James Madison", ...],
    document_types: ["convention_notes", ...]
  }
}
```

#### `currentResults` (Array)
Latest search results with scoring data.

```javascript
[
  {
    chunk: {...},      // Chunk object
    score: 5,          // Match count
    relevance: 12.3    // Relevance score
  },
  ...
]
```

#### `currentFilters` (Object)
Currently active filters.

```javascript
{
  collections: ["federalist_papers"],
  clauses: ["I.1", "II.2"],
  issues: ["federalism"]
}
```

### Methods

#### `init()`
Initialize the search engine by loading corpus and index files.

**Returns:** `Promise<boolean>`

**Throws:** Error if data files not found

**Example:**
```javascript
try {
  await searchEngine.init();
  console.log('Engine ready');
} catch (error) {
  console.error('Failed to initialize:', error);
}
```

#### `tokenizeQuery(query)`
Convert search query string into tokens.

**Parameters:**
- `query` (string): Search query

**Returns:** `Array<string>` - Array of lowercase tokens, stop words removed

**Example:**
```javascript
searchEngine.tokenizeQuery("what is the federal power?");
// Returns: ["federal", "power"]
```

#### `search(query, filters={})`
Search corpus with optional filters.

**Parameters:**
- `query` (string): Search query (required)
- `filters` (Object): Optional filters
  - `filters.collections` (Array<string>): Filter by collection names
  - `filters.clauses` (Array<string>): Filter by constitutional clauses
  - `filters.issues` (Array<string>): Filter by issue tags

**Returns:** `Array<Object>` - Results with chunk data and relevance scores

**Example:**
```javascript
const results = searchEngine.search("commerce power", {
  clauses: ["I.8"],
  issues: ["commerce"]
});

results.forEach(r => {
  console.log(r.chunk.title, r.relevance);
});
```

#### `getChunkById(chunkId)`
Retrieve a chunk by its ID.

**Parameters:**
- `chunkId` (string): Chunk ID

**Returns:** `Object|undefined` - Chunk object or undefined if not found

**Example:**
```javascript
const chunk = searchEngine.getChunkById("const_original_1787_0001");
console.log(chunk.text);
```

#### `getChunkByIndex(index)`
Retrieve a chunk by its position in the corpus array.

**Parameters:**
- `index` (integer): Array index (0-based)

**Returns:** `Object|undefined` - Chunk object

#### `getChunkIndex(chunkId)`
Get the array index of a chunk by its ID.

**Parameters:**
- `chunkId` (string): Chunk ID

**Returns:** `integer` - Array index, or -1 if not found

#### `getNextChunk(chunkId)`
Get the next chunk in the corpus (sequential, regardless of document).

**Parameters:**
- `chunkId` (string): Current chunk ID

**Returns:** `Object|undefined` - Next chunk or undefined if at end

#### `getPreviousChunk(chunkId)`
Get the previous chunk in the corpus.

**Parameters:**
- `chunkId` (string): Current chunk ID

**Returns:** `Object|undefined` - Previous chunk or undefined if at start

#### `getCollections()`
Get list of all unique collections in corpus.

**Returns:** `Array<string>` - Collection names

**Example:**
```javascript
const collections = searchEngine.getCollections();
// ["constitution", "madisonsnotes", "federalist_papers", ...]
```

#### `getClauses()`
Get list of all constitutional clause tags in corpus.

**Returns:** `Array<string>` - Clause IDs

#### `getIssues()`
Get list of all issue tags in corpus.

**Returns:** `Array<string>` - Issue tag IDs

#### `getClauseDisplayName(clauseId)`
Convert clause ID to human-readable display name.

**Parameters:**
- `clauseId` (string): Clause ID (e.g., "I.1")

**Returns:** `string` - Display name (e.g., "Legislative Power")

**Example:**
```javascript
searchEngine.getClauseDisplayName("II.2");
// Returns: "Commander in Chief"
```

#### `getIssueDisplayName(issueId)`
Convert issue tag ID to human-readable display name.

**Parameters:**
- `issueId` (string): Issue tag ID

**Returns:** `string` - Display name

**Example:**
```javascript
searchEngine.getIssueDisplayName("separation_of_powers");
// Returns: "Separation of Powers"
```

#### `calculateRelevance(chunk, tokens)`
Calculate relevance score for a chunk given search tokens.

**Parameters:**
- `chunk` (Object): Chunk object
- `tokens` (Array<string>): Search tokens

**Returns:** `number` - Relevance score

#### `exportChunksToJSON(chunks)`
Convert chunks array to JSON string.

**Parameters:**
- `chunks` (Array): Chunk objects to export

**Returns:** `string` - JSON-formatted string

**Example:**
```javascript
const json = searchEngine.exportChunksToJSON(results.map(r => r.chunk));
const blob = new Blob([json], {type: 'application/json'});
```

#### `exportChunksToCSV(chunks)`
Convert chunks array to CSV format.

**Parameters:**
- `chunks` (Array): Chunk objects to export

**Returns:** `string` - CSV-formatted string

**Example:**
```javascript
const csv = searchEngine.exportChunksToCSV(results.map(r => r.chunk));
// Columns: chunk_id, document_id, title, author, date, ...
```

## UI Class

Handles all user interface interactions and DOM rendering.

### Initialization

```javascript
// Created automatically when search engine is ready
// Event: 'searchEngineReady'

document.addEventListener('searchEngineReady', (event) => {
  const engine = event.detail;
  // UI is automatically initialized
});
```

### Public Methods

#### `performSearch()`
Execute search based on current search input and filters.

**Triggers:** Results rendering

#### `renderResults(results, query)`
Render search results to DOM.

**Parameters:**
- `results` (Array): Results from search engine
- `query` (string): Original search query

#### `renderFilters()`
Populate filter options from search engine data.

#### `showPassage(chunkId)`
Open passage viewer modal for a specific chunk.

**Parameters:**
- `chunkId` (string): Chunk ID to display

#### `closePassageViewer()`
Close the passage viewer modal.

#### `showPreviousChunk()`
Navigate to previous chunk in passage viewer.

#### `showNextChunk()`
Navigate to next chunk in passage viewer.

#### `exportCurrentChunk()`
Export currently displayed chunk as JSON file.

#### `clearFilters()`
Clear all active filters and reset to initial state.

#### `showError(message)`
Display error message to user.

**Parameters:**
- `message` (string): Error message text

#### `showMessage(message)`
Display temporary success/info message.

**Parameters:**
- `message` (string): Message text

#### `escapeHtml(text)`
Escape HTML special characters in text.

**Parameters:**
- `text` (string): Text to escape

**Returns:** `string` - HTML-escaped text

## Global Events

### 'searchEngineReady'
Fired when search engine is initialized and ready.

**Detail:** Search engine instance

```javascript
document.addEventListener('searchEngineReady', (event) => {
  const engine = event.detail;
  console.log(`Engine ready: ${engine.corpus.metadata.total_chunks} chunks`);
});
```

### 'searchEngineError'
Fired if search engine initialization fails.

**Detail:** Error message string

```javascript
document.addEventListener('searchEngineError', (event) => {
  const error = event.detail;
  console.error('Search engine error:', error);
});
```

## Data File Locations

### Relative to `frontend/index.html`:

- Corpus: `../data/chunks/constitution_full_corpus.json`
- Search Index: `../data/index/search_index.json`

### File Size Expectations

- Corpus: 2-4 MB (JSON)
- Search Index: <5 MB (JSON)
- **Total load:** ~5-8 MB
- **Compressed (gzip):** ~1 MB

## Performance Characteristics

- **Load time:** 2-5 seconds (typical)
- **Search time:** <100ms for most queries
- **Memory usage:** ~20-50 MB (depending on browser)

## Browser Compatibility

- **Chrome/Edge:** 60+
- **Firefox:** 55+
- **Safari:** 11+
- **Requires:** ES6 support, Fetch API

## Example Usage

```javascript
// Wait for engine to initialize
document.addEventListener('searchEngineReady', async (event) => {
  const engine = event.detail;

  // Simple search
  const results = engine.search("federalism");

  // Filtered search
  const fedResults = engine.search("power", {
    collections: ["federalist_papers"],
    issues: ["separation_of_powers"]
  });

  // Get specific chunk
  const chunk = engine.getChunkById("federalist_papers_full_0010");

  // Navigation
  const nextChunk = engine.getNextChunk(chunk.chunk_id);
  const prevChunk = engine.getPreviousChunk(chunk.chunk_id);

  // Export
  const csv = engine.exportChunksToCSV(results.map(r => r.chunk));
});
```

---

**Last Updated:** May 2, 2026
**API Version:** 1.0
