.PHONY: help setup build test clean dev phase1-init

PROJECT_DIR := $(shell pwd)
CARGO := cargo

help:
	@echo "Constitutional Research System - Rust/WASM Build Commands"
	@echo ""
	@echo "Setup & Development:"
	@echo "  make setup          - Initialize project structure"
	@echo "  make dev            - Open development session"
	@echo ""
	@echo "Building:"
	@echo "  make build          - Build all crates"
	@echo "  make build-lib      - Build core library only"
	@echo "  make build-server   - Build web server"
	@echo "  make build-cli      - Build CLI tool"
	@echo "  make build-wasm     - Build WASM frontend"
	@echo ""
	@echo "Testing & Validation:"
	@echo "  make test           - Run all tests"
	@echo "  make test-lib       - Test core library"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run clippy linter"
	@echo ""
	@echo "Cleanup:"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Phase 1 Development:"
	@echo "  make phase1-init    - Initialize Phase 1 structure"
	@echo "  make phase1-run     - Build and test Phase 1"
	@echo ""

setup:
	@echo "🚀 Setting up project structure..."
	@bash scripts/dev-setup.sh
	@echo "✅ Setup complete!"

dev:
	@echo "📂 Opening development session in: $(PROJECT_DIR)"
	@echo ""
	@echo "To open in a new terminal window:"
	@echo "  Terminal 1 (current): make watch"
	@echo "  Terminal 2 (new): cd $(PROJECT_DIR) && make build"
	@echo ""
	@bash -i

build:
	@echo "🔨 Building all crates..."
	$(CARGO) build --release

build-lib:
	@echo "📚 Building core library..."
	$(CARGO) build --release -p constitutional-lib

build-server:
	@echo "🌐 Building web server..."
	$(CARGO) build --release -p constitutional-server

build-cli:
	@echo "⚙️ Building CLI tool..."
	$(CARGO) build --release -p constitutional-cli

build-wasm:
	@echo "🕸️ Building WASM frontend..."
	@if command -v trunk >/dev/null 2>&1; then \
		cd crates/frontend && trunk build --release; \
	else \
		echo "❌ Trunk not found. Install with: cargo install trunk"; \
		exit 1; \
	fi

test:
	@echo "🧪 Running all tests..."
	$(CARGO) test --all

test-lib:
	@echo "🧪 Testing core library..."
	$(CARGO) test --lib -p constitutional-lib

watch:
	@echo "👁️ Watching for changes (rebuild on save)..."
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x "build --release" -x "test"; \
	else \
		echo "Install cargo-watch: cargo install cargo-watch"; \
		echo "Then run: cargo watch -x build -x test"; \
	fi

fmt:
	@echo "📝 Formatting code..."
	$(CARGO) fmt --all

lint:
	@echo "🔍 Running clippy linter..."
	$(CARGO) clippy --all --all-targets -- -D warnings

clean:
	@echo "🧹 Cleaning build artifacts..."
	$(CARGO) clean
	@rm -rf target/

phase1-init:
	@echo "📚 Initializing Phase 1: Core Libraries"
	@echo ""
	@echo "Creating core library structure..."
	@mkdir -p crates/lib/src
	@mkdir -p crates/lib/tests
	@mkdir -p crates/lib/benches
	@echo ""
	@echo "✅ Phase 1 structure ready!"
	@echo ""
	@echo "Next: make phase1-run"

phase1-run:
	@echo "🚀 Phase 1: Building core libraries..."
	$(CARGO) build --release -p constitutional-lib
	$(CARGO) test -p constitutional-lib
	@echo ""
	@echo "✅ Phase 1 build complete!"
	@echo ""
	@echo "Modules to implement:"
	@echo "  □ Tokenizer (tokenizer.rs)"
	@echo "  □ Full-Text Indexer (fulltext_index.rs)"
	@echo "  □ Fuzzy Matcher (fuzzy_match.rs)"
	@echo "  □ Vector Store (vector_store.rs)"
	@echo "  □ Chunking Engine (chunker.rs)"
	@echo "  □ Metadata Tagger (metadata_tagger.rs)"

info:
	@echo "📊 Project Information"
	@echo ""
	@echo "Rust Version:"
	@rustc --version
	@cargo --version
	@echo ""
	@echo "Project Root: $(PROJECT_DIR)"
	@echo ""
	@echo "Workspace Members:"
	@$(CARGO) metadata --format-version 1 | grep -oP '"name": "\K[^"]+' | sort | uniq
