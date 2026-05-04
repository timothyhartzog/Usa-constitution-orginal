#!/bin/bash
# Constitutional Research System - Rust/WASM Development Setup

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_DIR"

echo "🚀 Constitutional Research System - Rust/WASM Development Setup"
echo "================================================================"
echo ""

# Check Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Install from https://rustup.rs/"
    exit 1
fi

echo "✅ Rust toolchain found"
echo "   Rustc: $(rustc --version)"
echo "   Cargo: $(cargo --version)"
echo ""

# Create workspace structure if it doesn't exist
echo "📁 Setting up project structure..."

mkdir -p crates/lib/src
mkdir -p crates/server/src
mkdir -p crates/cli/src
mkdir -p crates/frontend/src
mkdir -p src-tauri/src
mkdir -p config
mkdir -p tests/fixtures

echo "✅ Directory structure ready"
echo ""

# Initialize workspace Cargo.toml if needed
if [ ! -f Cargo.toml ]; then
    echo "📝 Creating workspace Cargo.toml..."
    cat > Cargo.toml << 'EOF'
[workspace]
members = [
    "crates/lib",
    "crates/server",
    "crates/cli",
    "crates/frontend",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Constitutional Research Team"]
license = "MIT"

[workspace.dependencies]
tokio = { version = "1.35", features = ["full"] }
actix-web = "4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
log = "0.4"
env_logger = "0.11"
EOF
    echo "✅ Workspace created"
fi

echo ""
echo "🎯 Next steps:"
echo ""
echo "1. Start Phase 1 development:"
echo "   cd $PROJECT_DIR"
echo "   cargo new --lib crates/lib"
echo ""
echo "2. Open in IDE:"
echo "   code $PROJECT_DIR    # VS Code"
echo "   clion $PROJECT_DIR   # CLion"
echo ""
echo "3. Build and test:"
echo "   cargo build"
echo "   cargo test"
echo ""
echo "4. Open in new terminal:"
echo "   NEW_SESSION=1 bash -c 'cd $PROJECT_DIR && bash'"
echo ""
echo "================================================================"
echo "Setup complete! Ready to start Phase 1: Core Libraries"
echo "================================================================"
