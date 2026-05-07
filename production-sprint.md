# Constitutional Research System - Production Sprint Plan

**Project**: Rust/WASM Constitutional Research System  
**Status**: Planning Phase (Python/JS prototype complete, Rust/WASM rewrite planned)  
**Target**: Production-ready by end of Q3 2025  
**Effort**: ~10-12 weeks, 1-2 senior engineers

---

## Executive Summary

Convert the Constitutional Research System from Python (build) + JavaScript (frontend) to a production-grade Rust/WASM solution supporting:
- **Three deployment modes**: Static site, Web service (Actix), Desktop app (Tauri)
- **Three search engines**: Full-text (BM25), Fuzzy (BK-tree), Semantic (vectors)
- **Performance targets**: <10ms search queries, 60 FPS UI, <500KB WASM bundle
- **Dynamic ingestion**: Add/update documents via API with real-time indexing
- **Backward compatibility**: Export JSON for existing Python pipeline users

**Key Success Metrics**:
- ✓ All 2,565 Constitutional chunks searchable
- ✓ Search latency <100ms (typical queries)
- ✓ WASM bundle <500KB gzipped
- ✓ Desktop app ships for Windows/Mac/Linux
- ✓ Static site export matches Python output

---

## Tech Stack

### Backend
- **Language**: Rust (1.70+)
- **Web Framework**: Actix-web (HTTP/WebSocket)
- **Database**: SQLite (development), PostgreSQL (production)
- **Search Engines**: 
  - Full-text: Custom inverted index with BM25 scoring
  - Fuzzy: BK-tree (Burkhard-Keller tree)
  - Vector: Simple cosine similarity (HNSW optional Phase 6)
- **Serialization**: Bincode (binary), JSON (export)

### Frontend (WASM)
- **Framework**: Leptos (signals-based, <50KB compiled)
- **Bundler**: Trunk
- **HTTP Client**: gloo-net
- **CSS**: Tailwind CSS
- **Optional**: Mobile framework (React Native compatible later)

### Desktop
- **Framework**: Tauri (Rust + WebView)
- **Build Tool**: Cargo with Tauri CLI
- **IPC**: Rust → Frontend command binding

### CI/CD
- **VCS**: Git (github.com/timothyhartzog/usa-constitution-orginal)
- **CI**: GitHub Actions
- **Artifact Storage**: GitHub Releases
- **Static Hosting**: GitHub Pages or Vercel (for exported site)

---

## Phase Breakdown & Sprints

### Sprint 1: Foundation & Core Libraries (Weeks 1-2)
**Goal**: Establish Rust project structure and core indexing logic  
**Owner**: Backend lead

#### 1.1 Project Setup (Days 1-2)
- [ ] Create Cargo workspace (lib, server, cli, frontend, src-tauri)
- [ ] Set up GitHub Actions CI/CD pipeline
- [ ] Configure Rust toolchain (rustfmt, clippy)
- [ ] Add base dependencies (tokio, actix, serde, uuid, etc.)
- [ ] Create .github/workflows/rust.yml for tests + lint

**Deliverable**: Buildable Rust workspace, green CI

#### 1.2 Text Processing & Tokenization (Days 3-4)
**File**: `crates/lib/src/tokenizer.rs`

- [ ] Implement Tokenizer struct with stopword filtering
- [ ] Unicode normalization (lowercase, remove accents)
- [ ] Contraction handling ("don't" → ["do", "n't"])
- [ ] Min/max token length filtering
- [ ] Unit tests (edge cases: unicode, punctuation, numbers)

**Test Cases**:
- "The U.S. Constitution's preamble" → correct tokens
- "FEDERALIST No. 10" → ["federalist", "no", "10"]
- Empty/whitespace handling

**Deliverable**: Tokenizer with >95% test coverage, handles all edge cases

#### 1.3 Full-Text Index with BM25 (Days 5-7)
**File**: `crates/lib/src/fulltext_index.rs`

- [ ] Inverted index data structure (HashMap<term, Vec<chunk_id>>)
- [ ] BM25 scoring algorithm (TF-IDF enhancement)
- [ ] Multi-term AND queries (intersection of posting lists)
- [ ] Serialization to bincode format
- [ ] Unit tests: single term, multi-term, phrase queries

**Test Cases**:
- Query "Constitution" → correct chunks ranked by relevance
- Query "Constitution amendment" → chunks containing both, ranked
- Serialize/deserialize round-trip correctness

**Deliverable**: Full-text index with BM25 scoring, binary serialization

#### 1.4 Fuzzy Matching with BK-Tree (Days 8-9)
**File**: `crates/lib/src/fuzzy_match.rs`

- [ ] BK-tree construction from tokens
- [ ] Levenshtein distance calculation
- [ ] Fuzzy search with max_distance parameter
- [ ] Scoring by distance (higher score = closer match)
- [ ] Unit tests: typo tolerance, performance

**Test Cases**:
- Query "constituton" (typo) → finds "constitution"
- Distance threshold behavior
- Performance on 10K+ terms

**Deliverable**: BK-tree fuzzy matcher, edit distance tolerance

#### 1.5 Vector Store for Semantic Search (Days 10-11)
**File**: `crates/lib/src/vector_store.rs`

- [ ] VectorStore struct (chunk_id → Vec<f32> embeddings)
- [ ] Cosine similarity calculation
- [ ] Top-K retrieval
- [ ] Serialization/deserialization
- [ ] Unit tests: similarity math, ranking

**Test Cases**:
- Load pre-computed embeddings (from Sentence Transformers)
- Query "liberty" → semantically similar chunks
- Similarity scores normalized [0, 1]

**Deliverable**: Vector store with cosine similarity, compatible with external embeddings

#### 1.6 Chunking Engine (Days 12-13)
**File**: `crates/lib/src/chunker.rs`

- [ ] Port chunking strategies from Python (Constitution, Federalist, Jefferson, sliding window)
- [ ] Document type detection
- [ ] Chunk metadata (word_count, preview)
- [ ] Overlap handling (for sliding window)
- [ ] Unit tests: boundary conditions, metadata accuracy

**Test Cases**:
- Constitution chunk on Article boundaries
- Federalist split on "FEDERALIST No." markers
- Sliding window with 20% overlap

**Deliverable**: Chunker supporting 4+ strategies, tested against Python output

#### 1.7 Metadata Tagger (Days 14-15)
**File**: `crates/lib/src/metadata_tagger.rs`

- [ ] Load constitutional_clauses.json and issue_tags taxonomy
- [ ] Keyword matching (case-insensitive, boundary-aware)
- [ ] Tag scoring (higher weight for important clauses)
- [ ] Deduplication while preserving order
- [ ] Unit tests: matching accuracy, tag assignment

**Test Cases**:
- Chunk mentioning "First Amendment" → tagged ["I.1", "speech", "religion"]
- Multi-keyword matches (e.g., "due process")
- Taxonomy expansion safety (no false positives)

**Deliverable**: Metadata tagger with 90%+ accuracy on sample chunks

#### 1.8 Integration & Benchmarking (Days 16)
- [ ] Load existing data/chunks/constitution_full_corpus.json
- [ ] Build all indexes on 2,565 chunks
- [ ] Benchmark latencies (tokenization, indexing, search)
- [ ] Memory profiling
- [ ] Compare vs. Python pipeline performance

**Success Criteria**:
- Tokenize 2,565 chunks in <1 second
- Build full-text index in <2 seconds
- Single-term search in <1ms
- Multi-term search in <10ms
- Fuzzy search in <50ms

**Deliverable**: Benchmarking suite, performance baseline established

---

### Sprint 2: Database & Backend API (Weeks 3-4)
**Goal**: Standalone web service with REST API and WebSocket support  
**Owner**: Backend lead + API engineer

#### 2.1 Database Schema & Migrations (Days 1-2)
**File**: `crates/lib/src/db/schema.sql`

- [ ] Design SQLite schema (documents, chunks, embeddings, metadata)
- [ ] Create migration system (using `rusqlite` or `sqlx`)
- [ ] Indexes on frequently-queried columns (chunk_id, document_id, tags)
- [ ] Unit tests: schema correctness, migration ordering

**Tables**:
- `documents` (id, title, author, date, source_collection, source_url, document_type, metadata JSON)
- `chunks` (id, document_id, title, text, word_count, preview, issue_tags, clause_tags, created_at)
- `embeddings` (chunk_id, embedding BLOB, model_name, computed_at)
- `fulltext_terms` (term, chunk_id, frequency) for persistent FT index

**Deliverable**: Database schema with migrations, >95% test coverage

#### 2.2 Index Manager State (Days 3-4)
**File**: `crates/server/src/state.rs`

- [ ] IndexManager struct holding (FullTextIndex, FuzzyIndex, VectorStore, DbPool)
- [ ] RwLock for concurrent reads, exclusive writes
- [ ] Lazy initialization (load from disk on startup)
- [ ] Reindex methods (chunk, document, full corpus)
- [ ] Unit tests: state consistency, concurrent access

**Deliverable**: Thread-safe index state management

#### 2.3 REST API Handlers (Days 5-7)
**File**: `crates/server/src/handlers/`

Implement endpoints in Actix:

- [ ] `POST /api/search` - Query with type (fulltext|fuzzy|semantic), filters
  - Request: `{ query: String, search_type: SearchType, filters: FilterOptions }`
  - Response: `{ results: Vec<ChunkResult>, total_count: u32, elapsed_ms: u32 }`
  
- [ ] `GET /api/chunks/{id}` - Fetch chunk details
  - Response: Full chunk with text, metadata, related chunks
  
- [ ] `POST /api/documents` - Ingest document (chunk, index, broadcast update)
  - Request: `{ title, author, date, source_collection, document_type, text }`
  - Response: `{ chunk_ids: Vec<String>, indexed_at: Timestamp }`
  
- [ ] `GET /api/index` - Metadata (num_chunks, num_documents, tags, term_count)
  - Response: Index statistics for frontend UI
  
- [ ] `POST /api/export` - Export indexes as JSON
  - Response: JSON file download
  
- [ ] Error handling (ApiError enum with proper HTTP status codes)

**Test Cases**:
- Search returns results ranked correctly
- Document ingestion triggers reindexing
- Filtering by document, author, tag works
- Concurrent search requests handled correctly

**Deliverable**: 6+ API endpoints, integration tests, <100ms latency

#### 2.4 WebSocket for Live Updates (Days 8-9)
**File**: `crates/server/src/handlers/ws.rs`

- [ ] WebSocket server endpoint `/ws`
- [ ] Client message types (Subscribe, Unsubscribe)
- [ ] Server broadcast on index updates (new document, reindex complete)
- [ ] Client reconnection handling
- [ ] Unit tests: subscription, broadcasts, disconnect

**Message Types**:
- Server → Client: `{ type: "index_updated", chunks_added: u32, timestamp: i64 }`
- Client → Server: `{ type: "subscribe" }`

**Deliverable**: WebSocket endpoint, broadcast updates on document ingestion

#### 2.5 Document Ingestion Pipeline (Days 10-12)
**File**: `crates/server/src/ingest.rs`

- [ ] Load document from request (text blob)
- [ ] Chunk using appropriate strategy
- [ ] Tag metadata (issues, clauses)
- [ ] Build embeddings (call external API or skip for MVP)
- [ ] Write to database
- [ ] Update all indexes (FT, fuzzy, vector)
- [ ] Broadcast WebSocket update
- [ ] Background task for embeddings (async)

**Deliverable**: Full ingestion pipeline with async background tasks

#### 2.6 Integration Tests (Days 13-14)
- [ ] Test end-to-end: ingest → chunk → index → search
- [ ] Test concurrent searches during indexing
- [ ] Test WebSocket updates for multiple clients
- [ ] Test error scenarios (invalid input, DB failure)
- [ ] Load test: 100 concurrent searches

**Deliverable**: Integration test suite, >80% backend code coverage

**Success Criteria**:
- Server starts in <2 seconds
- API responds to all endpoints within SLA (<100ms for search)
- WebSocket broadcasts updates reliably
- 100 concurrent searches handled without drops

---

### Sprint 3: WASM Frontend (Weeks 5-6)
**Goal**: Interactive browser-based UI with real-time search  
**Owner**: Frontend lead

#### 3.1 Leptos Project Setup (Days 1-2)
**File**: `crates/frontend/Cargo.toml`, `crates/frontend/Trunk.toml`

- [ ] Initialize Leptos project with Trunk
- [ ] Configure WASM target (wasm32-unknown-unknown)
- [ ] Set up Tailwind CSS
- [ ] Dev server with hot reload
- [ ] Build pipeline (release = optimized)

**Deliverable**: Buildable Leptos app, dev server running

#### 3.2 Search Bar & Filter UI (Days 3-4)
**File**: `crates/frontend/src/components/search_bar.rs`, `filter_panel.rs`

- [ ] Search input with debounce (300ms)
- [ ] Filter panel (document, author, issue tag, clause tag)
- [ ] Search type selector (fulltext | fuzzy | semantic)
- [ ] Real-time filter preview ("X results")
- [ ] Responsive design (mobile-first)

**Deliverable**: Functional search UI, no backend integration yet

#### 3.3 Results Display & Pagination (Days 5-6)
**File**: `crates/frontend/src/components/results_list.rs`, `result_card.rs`

- [ ] Results list with pagination (10-50 items per page)
- [ ] Result card showing: title, author, date, preview, relevance score
- [ ] Click to expand/modal
- [ ] Infinite scroll or page controls
- [ ] Empty state ("No results found")

**Deliverable**: Results UI with pagination

#### 3.4 Modal Viewer for Full Chunks (Days 7-8)
**File**: `crates/frontend/src/components/modal.rs`

- [ ] Full-text display with syntax highlighting
- [ ] Metadata panel (document, author, tags)
- [ ] Related chunks (semantic similarity)
- [ ] Copy text, source link
- [ ] Print/export button
- [ ] Keyboard navigation (Escape to close, arrow keys for next/prev)

**Deliverable**: Modal with rich chunk display

#### 3.5 API Client Integration (Days 9-10)
**File**: `crates/frontend/src/api.rs`

- [ ] HTTP client using gloo-net
- [ ] Fetch /api/index on startup
- [ ] POST /api/search with query + filters
- [ ] GET /api/chunks/{id} for modal details
- [ ] Error handling & loading states
- [ ] Request caching (LRU)

**Deliverable**: Working API client, search integration

#### 3.6 WebSocket Integration (Days 11-12)
**File**: `crates/frontend/src/components/connection_status.rs`

- [ ] WebSocket connection on app load
- [ ] Listen for "index_updated" events
- [ ] Display connection status (green/red indicator)
- [ ] Auto-reconnect on disconnect
- [ ] Refresh index metadata when updated

**Deliverable**: Live connection indicator, auto-refresh on updates

#### 3.7 Export Functionality (Days 13)
**File**: `crates/frontend/src/components/export_panel.rs`

- [ ] Export selected results as JSON
- [ ] Export as CSV (simplified format)
- [ ] Download button with filename
- [ ] Progress indicator for large exports

**Deliverable**: Working export feature

#### 3.8 Styling & Responsive Design (Days 14)
- [ ] Tailwind CSS full theme
- [ ] Mobile breakpoints (sm, md, lg, xl)
- [ ] Dark mode toggle (optional)
- [ ] Accessibility (ARIA labels, semantic HTML)
- [ ] Print-friendly styles

**Deliverable**: Polished, responsive UI

#### 3.9 Frontend Tests & Optimization (Days 15)
- [ ] Component unit tests (search bar, results list, modal)
- [ ] Integration tests (search flow, export flow)
- [ ] Bundle size analysis (target <500KB gzipped)
- [ ] Performance profiling (Lighthouse)
- [ ] Cross-browser testing (Chrome, Firefox, Safari)

**Success Criteria**:
- WASM bundle <500KB gzipped
- Lighthouse score >90 (performance, accessibility)
- Search returns results in <500ms (including API latency)
- Mobile responsive on all screen sizes
- All components unit tested

**Deliverable**: Complete, tested WASM frontend with <500KB bundle

---

### Sprint 4: Desktop App (Tauri) (Weeks 7-8)
**Goal**: Standalone desktop application for Windows/Mac/Linux  
**Owner**: Desktop lead

#### 4.1 Tauri Project Setup (Days 1-2)
**File**: `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`

- [ ] Initialize Tauri project
- [ ] Configure build pipeline (Frontend → WASM → WebView)
- [ ] Icon assets (256x256 PNG, squared)
- [ ] App configuration (name, version, author)
- [ ] Cross-platform target setup

**Deliverable**: Buildable Tauri app skeleton

#### 4.2 IPC Command Bindings (Days 3-4)
**File**: `src-tauri/src/main.rs`

- [ ] Define Tauri commands (Rust → Frontend)
  - `search(query: String, filters: SearchFilters) -> Vec<ChunkResult>`
  - `fetch_chunk(id: String) -> Chunk`
  - `ingest_document(doc: Document) -> Result<Vec<String>>`
  - `export_results(chunks: Vec<String>, format: String) -> Result<PathBuf>`
  
- [ ] Set up index manager in app state
- [ ] Error serialization to JSON
- [ ] Async command handling

**Deliverable**: Working IPC bridge between Rust and frontend

#### 4.3 Frontend Integration (Days 5-6)
**File**: `crates/frontend/src/api.rs` (Tauri mode)

- [ ] Detect if running in Tauri (use IPC instead of HTTP)
- [ ] Create Tauri-specific API client
- [ ] Fall back to HTTP for web mode
- [ ] Share interface between web and desktop

**Deliverable**: Unified frontend working on web and desktop

#### 4.4 Desktop-Specific Features (Days 7-8)
- [ ] File picker for document import
- [ ] Local file explorer integration
- [ ] Offline indicator
- [ ] Update checker (GitHub releases)
- [ ] Keyboard shortcuts (Cmd+Q to quit, Cmd+, for settings)

**Deliverable**: Desktop UX enhancements

#### 4.5 Packaging & Distribution (Days 9-10)
- [ ] Build installers for Windows (.msi), Mac (.dmg), Linux (.deb, .AppImage)
- [ ] Code signing (certificates for Windows/Mac)
- [ ] GitHub Actions workflow for automated builds
- [ ] Release to GitHub Releases with checksums
- [ ] Documentation for users (download links, system requirements)

**Deliverable**: Signed installers, automated CI/CD

#### 4.6 Cross-Platform Testing (Days 11-12)
- [ ] Windows 10/11 testing
- [ ] macOS 11+ testing
- [ ] Linux (Ubuntu 20.04+) testing
- [ ] Performance profiling (startup time, memory usage)
- [ ] File system operations (permissions, paths)

**Success Criteria**:
- App launches in <3 seconds
- All features work identically to web version
- Binary size <80MB
- Signed and distributable

**Deliverable**: Production-ready desktop apps for 3 platforms

---

### Sprint 5: Static Site Export (Weeks 9)
**Goal**: CLI tool for exporting indexes for static hosting  
**Owner**: Backend lead

#### 5.1 Index Export Logic (Days 1-3)
**File**: `crates/lib/src/export.rs`

- [ ] Export FullTextIndex to JSON (inverted_index, term_frequencies, doc_lengths)
- [ ] Export FuzzyIndex to JSON (BK-tree structure or flattened terms)
- [ ] Export VectorStore to JSON (embeddings, dimensions)
- [ ] Export metadata (num_chunks, num_documents, tags, dates)
- [ ] Format matches existing Python pipeline output (for compatibility)

**Deliverable**: Export functions producing JSON indexes

#### 5.2 CLI Tool for Building Static Site (Days 4-5)
**File**: `crates/cli/src/main.rs`

Commands:
- `cargo run --bin build -- --input data/clean --output data/indexes`
  - Ingest → chunk → index → export pipeline
  - Progress logging
  - Export to static/data/*.json

- `cargo run --bin export -- --input data/indexes --output static/data`
  - Already-built indexes → JSON for static site

**Deliverable**: CLI tool with ingest and export commands

#### 5.3 Static Frontend (Vanilla JS) (Days 6-7)
**File**: `frontend/static/index.html`, `frontend/static/search.js`

- [ ] Load JSON indexes on page load
- [ ] Search implementation (client-side)
- [ ] Filter functionality (same as WASM frontend)
- [ ] No external dependencies (vanilla JS)
- [ ] Export functionality

**Deliverable**: Static HTML/JS that works with exported indexes

#### 5.4 Integration & Validation (Day 8)
- [ ] Export indexes using CLI
- [ ] Validate JSON against schema
- [ ] Test static frontend with exported data
- [ ] Compare vs. Python pipeline output (byte comparison where possible)
- [ ] Verify feature parity (search, filters, export)

**Success Criteria**:
- Exported JSON indexes are valid and loadable
- Static frontend works identically to WASM frontend
- File sizes comparable to Python version
- All 2,565 chunks present and searchable

**Deliverable**: Working static site, exportable via CLI

---

### Sprint 6: QA, Performance, & Launch (Weeks 10)
**Goal**: Production readiness, performance optimization, launch prep  
**Owner**: QA lead + DevOps engineer

#### 6.1 Performance Optimization (Days 1-3)
- [ ] Profile all search paths (flamegraph)
- [ ] Optimize hot paths (tokenization, BM25 scoring)
- [ ] Benchmark WASM bundle size (target <500KB)
- [ ] Test on slow networks (3G simulation)
- [ ] Memory profiling (heap allocations, peak usage)

**Targets**:
- Single-term query: <5ms
- Multi-term query: <20ms
- Fuzzy search: <50ms
- WASM load: <2 seconds on 3G

**Deliverable**: Performance profile, optimization report

#### 6.2 Security Audit (Days 4-5)
- [ ] Dependency audit (cargo audit)
- [ ] Input validation (SQL injection, XSS prevention)
- [ ] CORS configuration for API
- [ ] Rate limiting on search endpoint
- [ ] Security headers (CSP, X-Frame-Options, etc.)

**Deliverable**: Security audit report, fixes applied

#### 6.3 Documentation (Days 6-7)
- [ ] User guide (search features, export, keyboard shortcuts)
- [ ] API documentation (OpenAPI/Swagger spec)
- [ ] Deployment guide (server setup, env config)
- [ ] Developer guide (architecture, contributing)
- [ ] FAQ (common issues, performance tips)

**Deliverable**: Complete user + developer documentation

#### 6.4 Release Preparation (Days 8-9)
- [ ] Version bump (v1.0.0)
- [ ] CHANGELOG.md with feature list
- [ ] GitHub release notes
- [ ] Website/landing page
- [ ] Social media announcement (if applicable)

**Deliverable**: Release artifacts, marketing collateral

#### 6.5 Regression Testing (Days 10)
- [ ] Full test suite run (unit + integration + e2e)
- [ ] Compatibility testing (data import from Python pipeline)
- [ ] User acceptance testing (manual QA)
- [ ] Load testing (1000 concurrent searches)

**Success Criteria**:
- 100% test suite passing
- Zero critical bugs
- Performance meets all targets
- Load test: 1000 concurrent searches <200ms p99

**Deliverable**: Signed-off production release

---

## Sprint Milestones & Timeline

```
Sprint 1: Foundation (Weeks 1-2)          ✓ Tokenizer, indexes, chunking
Sprint 2: Backend API (Weeks 3-4)         ✓ Database, REST API, WebSocket
Sprint 3: WASM Frontend (Weeks 5-6)       ✓ Leptos UI, search integration
Sprint 4: Desktop App (Weeks 7-8)         ✓ Tauri binaries, IPC
Sprint 5: Static Export (Week 9)          ✓ CLI tool, static site
Sprint 6: QA & Launch (Week 10)           ✓ Performance, security, release

Total: 10 weeks (50 working days)
Contingency: 1-2 weeks buffer for blockers
```

---

## Acceptance Criteria by Phase

### Sprint 1 Complete When:
- [ ] All 7 library modules (tokenizer, FT, fuzzy, vector, chunker, tagger, export) compiling and tested
- [ ] 2,565 chunks processed successfully (match Python pipeline output)
- [ ] Benchmark targets met (tokenize <1s, search <10ms)
- [ ] >90% test coverage on core libs

### Sprint 2 Complete When:
- [ ] Server starts without errors, listens on :8080
- [ ] All 6+ API endpoints responding correctly
- [ ] WebSocket broadcasts updates to 10+ concurrent clients
- [ ] Document ingestion pipeline tested end-to-end
- [ ] Integration tests passing with >80% coverage

### Sprint 3 Complete When:
- [ ] WASM bundle <500KB gzipped
- [ ] Search returns results in <500ms (API latency included)
- [ ] UI responsive on mobile (tested on iPhone, Android)
- [ ] Export functionality working (JSON/CSV)
- [ ] Lighthouse score >90

### Sprint 4 Complete When:
- [ ] Desktop app builds for Windows, Mac, Linux
- [ ] All features working on desktop (IPC integration)
- [ ] Startup time <3 seconds
- [ ] Installers signed and distributable

### Sprint 5 Complete When:
- [ ] Static site exports and loads successfully
- [ ] Feature parity with Python pipeline (search, filters, export)
- [ ] JSON indexes validated against schema
- [ ] File sizes comparable to Python version

### Sprint 6 Complete When:
- [ ] All tests passing (unit, integration, e2e, performance)
- [ ] Security audit complete, no critical issues
- [ ] Documentation complete (user + dev guides)
- [ ] Load test successful (1000 concurrent searches)
- [ ] v1.0.0 released on GitHub

---

## Resource Requirements

### Team Composition
- **1 Backend/Rust Engineer** (Weeks 1-9)
  - Core libraries (Sprint 1)
  - Backend API (Sprint 2)
  - Static export (Sprint 5)
  - Performance optimization (Sprint 6)

- **1 Frontend Engineer** (Weeks 5-10)
  - WASM frontend (Sprint 3)
  - Desktop integration (Sprint 4)
  - Static site (Sprint 5)

- **1 DevOps/QA Engineer** (Weeks 1, 10)
  - CI/CD setup (Sprint 1, parallel)
  - Testing & release (Sprint 6)

- **Optional: Product Manager** (Weeks 1, 10)
  - Requirements refinement
  - Launch coordination

### Infrastructure
- **Development**: Local Rust toolchain, Node.js (for Trunk)
- **CI/CD**: GitHub Actions (free tier sufficient)
- **Staging**: Single Linux server (8GB RAM, 100GB SSD)
- **Production**: Scalable (e.g., AWS EC2 t3.large or equivalent)

### External Tools/Services
- **GitHub**: VCS, CI/CD, releases
- **Optional**: Sentry (error tracking), Datadog (monitoring)
- **Optional**: Sentence Transformers API (for embeddings, e.g., HuggingFace Inference API)

---

## Risk Mitigation

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| WASM bundle size bloat | Slow load time | Medium | Frequent size profiling, Leptos optimization, tree-shaking |
| Fuzzy search perf degradation at scale | >100ms on large corpus | Low | BK-tree algorithm proven; HNSW fallback in Phase 6 |
| WebSocket scalability (100+ clients) | Connection drops | Low | Use Actix-web connection pooling; load test early |
| Cross-platform Tauri issues | Desktop app broken on Mac/Linux | Medium | Early cross-platform testing (Week 7), CI/CD builds for all platforms |
| Data migration from Python | Incompatibility | Low | Extensive compatibility tests (Sprint 6); keep Python pipeline runnable as fallback |
| Embedding API outages | Semantic search unavailable | Low | Graceful degradation (fall back to FT search); optional feature |
| Team turnover | Knowledge loss | Low | Document architecture; pair programming on critical paths |

---

## Definition of Done (DoD)

Each sprint is complete when:
1. **Code**: All stories implemented, reviewed, merged to `main`
2. **Tests**: Unit + integration tests passing, >80% coverage
3. **Docs**: Code comments, API docs, and task tracking updated
4. **Performance**: Benchmarks meet targets; profiling done
5. **Security**: No critical vulnerabilities (cargo audit clean)
6. **Demo-ready**: Feature demoed to stakeholders

---

## Deployment Strategy

### Phase 1: Internal Testing (Week 1-8)
- Continuous deployment to staging via GitHub Actions
- Nightly builds for all platforms (desktop)
- Manual QA testing on staging

### Phase 2: Beta Release (Week 9)
- Tag as v1.0.0-beta1 on GitHub
- Limit distribution to early testers (10-20 users)
- Collect feedback, iterate

### Phase 3: General Availability (Week 10)
- Promote v1.0.0 release
- Publish to app stores / distribution channels
- Setup monitoring (error tracking, performance metrics)

### Rollback Plan
- Keep Python pipeline operational as fallback
- GitHub releases with version pinning
- Database backups before major schema changes

---

## Monitoring & Observability (Post-Launch)

- **Error Tracking**: Sentry for frontend + backend
- **Metrics**: Prometheus + Grafana (search latency, indexing time)
- **Logs**: Structured logging (tracing crate)
- **User Analytics**: Optional (privacy-respecting telemetry)
- **Alerts**: Critical errors (indexing failure, API errors >1% of requests)

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Search latency (p50) | <5ms | APM / profiling |
| Search latency (p99) | <100ms | APM / profiling |
| WASM bundle size | <500KB (gzipped) | Build output |
| Desktop app startup | <3 seconds | Manual timing |
| Test coverage | >85% | Code coverage tool |
| Uptime (server) | >99.5% | Monitoring |
| User satisfaction | 4.5+/5 stars | Feedback survey |

---

## Post-MVP Features (Phase 6+)

Once core MVP is production-ready:
- Semantic search with HNSW index (sub-linear on 1M+ chunks)
- RAG integration (Claude API for Q&A)
- User accounts & bookmarks
- Collaborative annotations
- Mobile app (React Native)
- Multi-language support
- Diff highlighting for source versions

---

## Sign-Off

- **Project Lead**: [ ] Review & approve
- **Backend Lead**: [ ] Agree on timelines
- **Frontend Lead**: [ ] Agree on design/UX
- **QA Lead**: [ ] Testing strategy approved

---

## Appendix: References

- **Architecture**: See `RUST_WASM_PLAN.md` for detailed system design
- **Current Data**: `data/chunks/constitution_full_corpus.json` (2,565 chunks)
- **Config**: `config/sources_manifest.json`, `config/constitutional_clauses.json`
- **Existing Frontend**: `frontend/` (JavaScript, refactor to Leptos)
- **GitHub**: https://github.com/timothyhartzog/usa-constitution-orginal
