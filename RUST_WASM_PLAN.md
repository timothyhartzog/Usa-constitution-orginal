# Rust/WASM Constitutional Research System - Detailed Implementation Plan

## Context

**Why**: User wants to move the Constitutional Research System from Python (build pipeline) + JavaScript (frontend) to a full-stack Rust/WASM solution. Goals are:
- **New features**: Fuzzy search, semantic/vector search, live document updates, dynamic ingestion
- **Extreme performance**: Microsecond-level search, support millions of chunks, sub-millisecond response times
- **Multiple deployment targets**: Static WASM site, web server + WASM frontend, desktop application (Tauri)
- **Advanced capability**: Enable RAG, AI integration, user features, collaborative annotation

**Current State**: 
- Python pipeline (ingest → clean → chunk → index) produces static JSON indexes
- JavaScript frontend provides client-side full-text search on 2,565 chunks
- ~25.5 MB total data loaded into browser
- Performance bottlenecks: O(n²) array intersection, regex recompilation, full DOM re-render

**Outcome**: Production-ready Rust/WASM system supporting fuzzy search, vector similarity, dynamic document ingestion, and deployment as static site, web service, or desktop app.

---

## Architecture Overview

### High-Level System Design

```
┌─────────────────────────────────────────────────────────────────┐
│                        Rust Backend (Web Service)                │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐   │
│  │ Document Ingestion│  │  Index Manager    │  │   Database   │   │
│  │ & Processing      │  │ (Full-Text,      │  │ (SQLite/Pg)  │   │
│  │ (Tokenization,    │  │  Vector,Fuzzy)   │  │              │   │
│  │  Dedup, Clean)    │  └──────────────────┘  └──────────────┘   │
│  └──────────────────┘                                             │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              HTTP API (Actix-web Framework)               │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │ • POST /api/search - Full-text, fuzzy, semantic search   │   │
│  │ • POST /api/documents - Add/update document              │   │
│  │ • GET /api/index - Fetch index metadata                  │   │
│  │ • WebSocket /ws - Live index updates                     │   │
│  │ • POST /api/batch-ingest - Bulk document upload          │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │              Shared Rust Libraries (lib.rs)               │   │
│  ├──────────────────────────────────────────────────────────┤   │
│  │ • Tokenizer (language-aware, stopword filtering)          │   │
│  │ • Full-Text Indexer (inverted index, term freq)           │   │
│  │ • Fuzzy Matcher (Levenshtein/BK-tree)                     │   │
│  │ • Vector Store (embeddings, similarity search)            │   │
│  │ • Chunking Engine (paragraph-aware, overlap)              │   │
│  │ • Metadata Tagger (keyword matching, taxonomy)            │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
         │                           │                      │
         │                           │                      │
    (Shared             (JSON export            (Desktop App
     Library)           for static site)         via Tauri)
         │                           │                      │
         ▼                           ▼                      ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  WASM Frontend   │  │  Static HTML/JS  │  │  Tauri Desktop   │
│  (Leptos or      │  │  (Next.js or     │  │  App             │
│   Yew)           │  │   plain HTML)    │  │  (Rust + WebView)│
│                  │  │                  │  │                  │
│  • React-like    │  │  • Fetches JSON  │  │  • Direct Rust   │
│    components    │  │    indexes       │  │    lib calls     │
│  • Real-time UI  │  │  • Client-side   │  │  • No network    │
│  • WebSocket     │  │    search        │  │    overhead      │
│    integration   │  │  • Works offline │  │  • Cross-platform│
└──────────────────┘  └──────────────────┘  └──────────────────┘
```

### Three Deployment Modes (Same Codebase)

1. **Static Site Mode**
   - Backend builds final JSON indexes (ingest → clean → chunk → index)
   - Exports to static files
   - Frontend loads via HTTP, searches client-side
   - No server dependency (CDN-friendly)

2. **Web Service Mode**
   - Full backend API running on server
   - WASM frontend communicates via HTTP/WebSocket
   - Server handles indexing, search, document management
   - Supports live document ingestion and index updates

3. **Desktop Mode (Tauri)**
   - Rust backend embedded in desktop app
   - WASM frontend via WebView
   - Direct IPC communication (no HTTP overhead)
   - Offline-first, all data local

---

## Phase 1: Core Libraries (Foundation)

### 1.1 Tokenizer & Text Processing
**File**: `src/lib/tokenizer.rs`

```rust
pub struct Tokenizer {
    stopwords: HashSet<String>,
    min_length: usize,
}

pub fn tokenize(text: &str) -> Vec<String>
pub fn normalize_whitespace(text: &str) -> String
pub fn slugify(text: &str) -> String
```

**Features**:
- Language-aware tokenization (handle contractions, punctuation)
- Configurable stopword list (58 common English words)
- Unicode support (handle accented characters, case-folding)
- Min/max token length filtering
- Duplicate removal while preserving order

### 1.2 Full-Text Indexer
**File**: `src/lib/fulltext_index.rs`

```rust
pub struct FullTextIndex {
    inverted_index: HashMap<String, Vec<ChunkId>>,
    term_frequencies: HashMap<(String, ChunkId), u32>,
    doc_lengths: HashMap<ChunkId, u32>,
}

pub fn add_chunk(&mut self, chunk_id: ChunkId, text: &str)
pub fn search(&self, query: &str) -> Vec<(ChunkId, f32)>  // BM25 scoring
pub fn serialize(&self) -> Vec<u8>
pub fn deserialize(data: &[u8]) -> Self
```

**Algorithms**:
- Inverted index for O(1) term lookup
- BM25 ranking for relevance (better than simple term frequency)
- Serialization to binary format (smaller than JSON, faster load)

### 1.3 Fuzzy Matcher
**File**: `src/lib/fuzzy_match.rs`

```rust
pub struct FuzzyIndex {
    bk_tree: BKTree<String>,  // Burkhard-Keller tree for metric search
    chunks: HashMap<ChunkId, Vec<String>>,
}

pub fn search(&self, query: &str, max_distance: usize) -> Vec<(ChunkId, f32)>
```

**Algorithm**: BK-Tree (Burkhard-Keller tree) for sub-linear fuzzy search
- Metric space indexing on Levenshtein distance
- O(log n) worst-case for single-term fuzzy search
- Supports "find typo-tolerant matches"

### 1.4 Vector Store (Semantic Search)
**File**: `src/lib/vector_store.rs`

```rust
pub struct VectorStore {
    embeddings: HashMap<ChunkId, Vec<f32>>,  // e.g., 384-dim embeddings
    dimension: usize,
}

pub fn add_embedding(&mut self, chunk_id: ChunkId, embedding: Vec<f32>)
pub fn cosine_similarity(&self, query_embedding: &[f32], k: usize) -> Vec<(ChunkId, f32)>
```

**Features**:
- Store dense vector embeddings (384-768 dimensions typical)
- Cosine similarity search
- Optional: HNSW (Hierarchical Navigable Small World) for sub-linear search on large corpora
- Embeddings computed offline (e.g., via Sentence Transformers) or via API

### 1.5 Chunking Engine
**File**: `src/lib/chunker.rs`

```rust
pub enum ChunkStrategy {
    ConstitutionSections,
    FederalistEssays,
    JeffersonLetters,
    SlidingWindow { target_words: usize, min: usize, max: usize, overlap: usize },
}

pub fn chunk_document(text: &str, strategy: ChunkStrategy) -> Vec<Chunk>
```

**Strategies** (ported from Python):
- Constitution-specific (split on ARTICLE markers)
- Federalist Essays (split on "FEDERALIST No.")
- Jefferson Letters (split on "TO {NAME}")
- Generic sliding window (configurable overlap, target size)

### 1.6 Metadata Tagger
**File**: `src/lib/metadata_tagger.rs`

```rust
pub struct MetadataTagger {
    constitutional_clauses: Vec<ConstitutionalClause>,
    issue_tags: Vec<IssueTag>,
}

pub fn tag_chunk(&self, chunk: &Chunk) -> (Vec<String>, Vec<String>)  // (issue_tags, clause_tags)
```

**Logic**:
- Keyword matching against taxonomy (constitutional_clauses.json)
- Higher threshold for important clauses (I.8, II.1)
- Preserve insertion order, deduplicate

---

## Phase 2: Backend Service

### 2.1 Database Schema
**File**: `src/db/schema.sql` (SQLite)

```sql
CREATE TABLE documents (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT,
    date TEXT,
    source_collection TEXT,
    source_url TEXT,
    document_type TEXT,
    ingested_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP,
    metadata JSON
);

CREATE TABLE chunks (
    id TEXT PRIMARY KEY,
    document_id TEXT NOT NULL REFERENCES documents(id),
    title TEXT,
    text TEXT NOT NULL,
    word_count INTEGER,
    preview TEXT,
    issue_tags TEXT,  -- JSON array or pipe-delimited
    clause_tags TEXT,
    created_at TIMESTAMP,
    FOREIGN KEY (document_id) REFERENCES documents(id)
);

CREATE TABLE embeddings (
    chunk_id TEXT PRIMARY KEY REFERENCES chunks(id),
    embedding BLOB,  -- Serialized f32 vector
    model_name TEXT,
    computed_at TIMESTAMP
);

CREATE TABLE fulltext_index (
    term TEXT NOT NULL,
    chunk_id TEXT NOT NULL REFERENCES chunks(id),
    frequency INTEGER,
    PRIMARY KEY (term, chunk_id)
);

CREATE TABLE fuzzy_index (
    -- BK-tree serialized as BLOB or separate structure
    data BLOB,
    updated_at TIMESTAMP
);
```

### 2.2 Web API (Actix-web)
**File**: `src/bin/server.rs`

```rust
#[post("/api/search")]
async fn search_handler(
    query: web::Json<SearchRequest>,
    index: web::Data<Arc<IndexState>>,
) -> Result<HttpResponse, ApiError>

#[post("/api/documents")]
async fn ingest_document_handler(
    doc: web::Json<DocumentInput>,
    db: web::Data<DbPool>,
) -> Result<HttpResponse, ApiError>

#[ws("/ws")]
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error>
```

**Endpoints**:
- `POST /api/search` - Search with query, filters, search_type (fulltext|fuzzy|semantic)
- `POST /api/documents` - Add/update document, triggers reindexing
- `GET /api/index` - Fetch index metadata (terms, documents, tags)
- `GET /api/documents/{id}` - Fetch chunk details
- `POST /api/batch-ingest` - Upload multiple documents
- `WebSocket /ws` - Live index updates (broadcast when docs added/indexed)
- `GET /api/export` - Export index as JSON for static site

### 2.3 Index Manager
**File**: `src/lib/index_manager.rs`

```rust
pub struct IndexManager {
    fulltext: Arc<RwLock<FullTextIndex>>,
    fuzzy: Arc<RwLock<FuzzyIndex>>,
    vector: Arc<RwLock<VectorStore>>,
    db: DbPool,
}

pub async fn ingest_document(&self, doc: Document) -> Result<()>
pub async fn reindex_chunk(&self, chunk_id: ChunkId) -> Result<()>
pub async fn search(&self, query: &str, search_type: SearchType) -> Result<Vec<Result>>
pub async fn export_index(&self) -> Result<IndexExport>
```

**Logic**:
- Lazy-load indexes from disk on startup
- Update fulltext + fuzzy indexes synchronously
- Update vector embeddings asynchronously (via background task)
- Broadcast WebSocket updates on changes

---

## Phase 3: Frontend (WASM)

### 3.1 Frontend Framework Choice
**Decision**: Use **Leptos** (Rust signals-based framework, like SolidJS/Svelte)
- Reactive signals (no virtual DOM)
- Server-side rendering capable (if needed later)
- Excellent WASM compilation, tiny bundle sizes
- Works with Tauri seamlessly

**Alternative**: Yew (more React-like, heavier but more familiar)

### 3.2 WASM Frontend Structure
**File**: `src/frontend/lib.rs` (Leptos app)

```rust
#[component]
pub fn App() -> impl IntoView {
    // Search bar, filters, results
}

#[component]
fn SearchBar(on_search: Callback<SearchRequest>) -> impl IntoView {}

#[component]
fn ResultsList(results: Vec<SearchResult>) -> impl IntoView {}

#[component]
fn ResultCard(result: SearchResult) -> impl IntoView {}

#[component]
fn Modal(chunk: Chunk) -> impl IntoView {}
```

### 3.3 API Client (WASM-safe)
**File**: `src/frontend/api.rs`

```rust
pub async fn search(query: &str, filters: SearchFilters) -> Result<Vec<SearchResult>>
pub async fn fetch_document(chunk_id: &str) -> Result<Chunk>
pub async fn subscribe_to_updates(callback: Closure) -> WebSocket
```

Uses `gloo-net` for HTTP and `web-sys` for WebSocket.

### 3.4 State Management
**File**: `src/frontend/state.rs`

```rust
#[derive(Clone)]
pub struct AppState {
    pub results: Signal<Vec<SearchResult>>,
    pub current_chunk: Signal<Option<Chunk>>,
    pub filters: Signal<SearchFilters>,
    pub search_term: Signal<String>,
    pub is_connected: Signal<bool>,  // WebSocket connection state
}
```

---

## Phase 4: Desktop App (Tauri)

### 4.1 Tauri Configuration
**File**: `src-tauri/tauri.conf.json`

```json
{
  "build": {
    "beforeDevCommand": "cd frontend && trunk serve",
    "beforeBuildCommand": "cd frontend && trunk build --release",
    "devPath": "http://localhost:8080",
    "frontendDist": "../target/dist"
  },
  "app": {
    "windows": [{
      "label": "main",
      "title": "Constitutional Research System"
    }]
  }
}
```

### 4.2 IPC Bridge (Rust ↔ Frontend)
**File**: `src-tauri/src/main.rs`

```rust
#[tauri::command]
async fn search(query: String, index: State<'_, Arc<IndexManager>>) -> Result<Vec<SearchResult>> {
    index.search(&query, SearchType::FullText).await
}

#[tauri::command]
async fn ingest_document(doc: Document, index: State<'_, Arc<IndexManager>>) -> Result<()> {
    index.ingest_document(doc).await
}
```

Direct Rust function calls from WebView (no HTTP overhead).

---

## Phase 5: Static Site Export

### 5.1 Index Exporter
**File**: `src/lib/export.rs`

```rust
pub fn export_to_static_indexes(index: &IndexManager) -> (
    IndexMetadata,
    InvertedIndex,
    FuzzyIndex,
    VectorIndex,
)

pub fn export_to_json(indexes: &ExportedIndexes) -> String
```

Creates the same JSON format as Python build pipeline (compatibility with existing frontend).

### 5.2 Build Tool
**File**: `src/bin/build.rs`

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Ingest documents from sources_manifest.json
    // 2. Clean and chunk
    // 3. Build indexes
    // 4. Export to JSON
    // 5. Copy to static/data/ directory
}
```

One-shot command: `cargo run --bin build`

---

## Implementation Phases & Milestones

### **Phase 1: Core Libraries** (1-2 weeks)
**Deliverable**: Rust library crate with all indexing logic

- [ ] Tokenizer (1-2 days)
- [ ] Full-text indexer with BM25 (2-3 days)
- [ ] Fuzzy matcher with BK-tree (2-3 days)
- [ ] Vector store + cosine similarity (1-2 days)
- [ ] Chunking engine (1-2 days)
- [ ] Metadata tagger (1 day)
- [ ] Serialization/deserialization (1 day)
- [ ] Unit tests for all modules (ongoing)

**Success Criteria**:
- Tokenizer handles 2,565 chunks correctly
- Full-text search ranks documents by BM25 (vs. simple term freq)
- Fuzzy search finds typos within edit distance 2
- Vector search returns semantically similar chunks
- All modules tested with Python pipeline output

---

### **Phase 2: Backend Server** (2-3 weeks)
**Deliverable**: Actix-web server with API and WebSocket support

- [ ] Database schema and migrations (1-2 days)
- [ ] Index manager state (1-2 days)
- [ ] REST API endpoints (2-3 days)
- [ ] WebSocket live updates (2 days)
- [ ] Document ingestion pipeline (2-3 days)
- [ ] Integration tests (2-3 days)

**Success Criteria**:
- Server starts and serves API requests
- Search returns results in <100ms for typical queries
- Documents can be added via API and indexed in real-time
- WebSocket broadcasts updates to connected clients

---

### **Phase 3: WASM Frontend** (2-3 weeks)
**Deliverable**: Interactive Leptos app running in browser

- [ ] Leptos project setup + Trunk bundler (1 day)
- [ ] Search bar + filter UI components (2-3 days)
- [ ] Results list and pagination (2 days)
- [ ] Modal viewer (1-2 days)
- [ ] API client (gloo-net) (1-2 days)
- [ ] WebSocket connection handler (1 day)
- [ ] Export functionality (JSON/CSV) (1 day)
- [ ] Mobile-responsive CSS (1-2 days)

**Success Criteria**:
- App loads in <2 seconds (WASM bundle <500KB)
- Search returns results instantly (WebSocket latency)
- UI is responsive on desktop and mobile
- Export works correctly

---

### **Phase 4: Desktop App (Tauri)** (1-2 weeks)
**Deliverable**: Standalone executable for Windows/Mac/Linux

- [ ] Tauri project setup (1 day)
- [ ] IPC command bindings (1-2 days)
- [ ] Frontend integration (1 day)
- [ ] Packaging + signing (1-2 days)
- [ ] Cross-platform testing (1-2 days)

**Success Criteria**:
- App builds and runs on Windows, Mac, Linux
- Search is instant (direct Rust calls, no HTTP)
- Binary size < 80MB

---

### **Phase 5: Static Site Export** (1 week)
**Deliverable**: CLI tool to export indexes for static hosting

- [ ] Index export logic (1-2 days)
- [ ] JSON serialization (1 day)
- [ ] Build CLI tool (1 day)
- [ ] Integration tests (1 day)

**Success Criteria**:
- Generated JSON indexes match Python pipeline output
- Static frontend (vanilla JS or Leptos hydration) works with exported indexes
- File sizes comparable to Python version

---

## Technical Decisions & Rationale

### Why Leptos Over Yew?
- Smaller bundle size (Leptos ~40KB vs Yew ~100KB+)
- Fine-grained reactivity (no virtual DOM diffing overhead)
- Better performance on large result lists
- Built-in server-side rendering (future-proof for static export)

### Why Actix-web?
- Fastest HTTP framework in Rust (consistent benchmarks)
- Excellent async/await support (tokio runtime)
- WebSocket support built-in
- Large ecosystem, well-documented
- Performance critical for extreme use case

### Why BK-Tree for Fuzzy Search?
- Metric space indexing reduces comparisons from O(n) to O(log n) average
- Supports dynamic insertion (add documents to fuzzy index live)
- Works with Levenshtein distance (edit distance)
- Single-term fuzzy queries very fast

### Why Separate Indexes (Fulltext, Fuzzy, Vector)?
- Different query semantics: "exact phrase" vs "typo-tolerant" vs "conceptually similar"
- Can optimize each independently
- User can specify search type
- Allows combining results (multi-mode search)

### Serialization Format (Bincode vs JSON)?
- **Bincode** for internal/production: Binary format, 10x smaller, faster load
- **JSON** for static export: Human-readable, compatible with JavaScript
- Load Bincode in server, export as JSON for static site

---

## File Structure

```
constitutional-research-system/
├── Cargo.toml                          # Workspace root
├── Cargo.lock
│
├── crates/
│   ├── lib/                            # Core libraries
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── tokenizer.rs
│   │       ├── fulltext_index.rs
│   │       ├── fuzzy_match.rs
│   │       ├── vector_store.rs
│   │       ├── chunker.rs
│   │       ├── metadata_tagger.rs
│   │       ├── export.rs
│   │       └── db/
│   │           ├── mod.rs
│   │           └── schema.sql
│   │
│   ├── server/                         # Web service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # Actix-web app
│   │       ├── handlers/
│   │       │   ├── search.rs
│   │       │   ├── documents.rs
│   │       │   ├── export.rs
│   │       │   └── ws.rs
│   │       ├── state.rs                # Index manager state
│   │       ├── error.rs                # Error types
│   │       └── db.rs                   # Database pool
│   │
│   ├── cli/                            # Build + export CLI
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs                 # CLI commands
│   │       ├── ingest.rs
│   │       ├── build.rs
│   │       └── export.rs
│   │
│   └── frontend/                       # WASM frontend
│       ├── Cargo.toml
│       ├── Trunk.toml
│       ├── index.html
│       └── src/
│           ├── lib.rs                  # Leptos app root
│           ├── components/
│           │   ├── search_bar.rs
│           │   ├── results_list.rs
│           │   ├── filter_panel.rs
│           │   └── modal.rs
│           ├── api.rs                  # HTTP/WebSocket client
│           ├── state.rs                # Signals & state
│           └── styles/
│               └── main.css
│
├── src-tauri/                          # Desktop app (Tauri)
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── src/
│   │   └── main.rs                     # Tauri app + IPC handlers
│   └── icons/
│
├── config/
│   ├── sources_manifest.json           # Document sources (reuse from Python)
│   └── constitutional_clauses.json     # Taxonomy (reuse)
│
└── tests/
    ├── integration_tests.rs            # Full-stack tests
    └── fixtures/                       # Sample data
```

---

## Critical Files to Reuse/Port

From existing Python system:
- `config/sources_manifest.json` - Document definitions
- `config/constitutional_clauses.json` - Constitutional taxonomy
- `data/raw/*` - Raw source documents (cache)
- `data/clean/*` - Cleaned text (input to new pipeline)

Porting priority:
1. Chunking strategies (from `scripts/chunk_documents.py`)
2. Metadata tagging logic (from `scripts/utils/metadata_tagging.py`)
3. Tokenization & normalization (from `scripts/utils/pipeline.py`)
4. Document taxonomy (JSON configs)

---

## Performance Goals & Optimization Strategies

### Search Performance Targets
- **Single-term query**: <1ms (inverted index lookup)
- **Multi-term query**: <10ms (intersection + ranking)
- **Fuzzy search**: <50ms (BK-tree search + scoring)
- **Semantic search**: <100ms (vector similarity)
- **UI render**: <16ms (60 FPS, ~60-100 results)

### Optimization Strategies

1. **Indexing** (Build-time)
   - Bincode serialization for fast memory load
   - Memory-mapped files for large indexes
   - Parallel chunking + metadata tagging

2. **Search** (Query-time)
   - Early termination (top-K results)
   - HNSW for vector search (sub-linear on large corpora)
   - Query caching (LRU cache for repeated searches)

3. **Frontend** (Render-time)
   - Leptos fine-grained reactivity (no full re-render)
   - Virtual scrolling for large result lists
   - Debounce search input (avoid redundant searches)
   - Progressive enhancement (quick summary, lazy-load full text)

---

## Verification & Testing Strategy

### Unit Tests
- Tokenizer: edge cases (unicode, contractions, stopwords)
- Full-text index: multi-term queries, scoring accuracy
- Fuzzy matcher: edit distance verification
- Vector store: cosine similarity math
- Chunker: paragraph boundaries, overlap calculation
- Metadata tagger: keyword matching

### Integration Tests
- End-to-end: ingest → chunk → index → search
- Search types: fulltext + fuzzy + semantic results match expectations
- API: CRUD operations, WebSocket broadcasts
- Export: generated JSON is valid, matches schema

### Performance Tests
- Benchmark search latency on 2,565 chunks
- Benchmark on larger synthetic corpus (10K, 100K, 1M chunks)
- Memory profiling (heap usage, peak allocations)
- Bundle size analysis (WASM gzip compression)

### Compatibility Tests
- Static site export vs. Python pipeline output (JSON schema)
- Frontend works with server API
- Tauri IPC communication
- Cross-platform desktop app (Windows/Mac/Linux)

---

## Phase 6: Advanced Features (Optional, Post-MVP)

Once core system is stable:

### 6.1 Semantic Search Enhancement
- Use pre-computed embeddings from Sentence Transformers
- Add HNSW index for sub-linear vector search on millions of chunks
- Support multi-modal queries (text + image embeddings later)

### 6.2 RAG Integration
- Integrate with Claude API (or other LLM)
- Retrieve top K chunks for a query
- Pass to LLM for question-answering, summarization
- Cache LLM responses for repeated queries

### 6.3 User Features
- User accounts & authentication (JWT, OAuth)
- Saved searches + bookmarks
- Annotation/highlighting with timestamps
- Export saved collections

### 6.4 Collaborative Features
- Real-time cursor presence (who's looking at what)
- Shared highlights + comments
- Collaborative annotations (like Google Docs)

---

## Summary

This plan builds a production-grade constitutional research system in Rust/WASM with:
- **3 deployment modes**: Static site, web service, desktop app (same codebase)
- **3 search engines**: Full-text (BM25), fuzzy (BK-tree), semantic (vectors)
- **Extreme performance**: Sub-millisecond search on millions of chunks
- **Dynamic ingestion**: Add documents live via API, instant indexing
- **Modern frontend**: Leptos WASM, reactive UI, WebSocket integration
- **Backward compatibility**: Exports JSON compatible with existing Python pipeline

Total estimated effort: **8-12 weeks** for core MVP (Phases 1-5), assuming one senior Rust developer and one frontend engineer.
