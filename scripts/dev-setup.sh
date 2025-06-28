#!/bin/bash

# Development setup script for Ordinator
set -e

echo "🚀 Setting up Ordinator development environment..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed. Please install Rust and Cargo first."
    exit 1
fi

echo "✅ Rust and Cargo are installed"

# Install development dependencies
echo "📦 Installing development dependencies..."

# Install cargo-watch for development
if ! command -v cargo-watch &> /dev/null; then
    echo "Installing cargo-watch..."
    cargo install cargo-watch
fi

# Install cargo-audit for security audits
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

# Install cargo-tarpaulin for code coverage (optional)
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "Installing cargo-tarpaulin for code coverage..."
    cargo install cargo-tarpaulin
fi

echo "✅ Development dependencies installed"

# Build the project
echo "🔨 Building Ordinator..."
cargo build

echo "✅ Build successful"

# Run tests
echo "🧪 Running tests..."
cargo test

echo "✅ Tests passed"

# Check code formatting
echo "🎨 Checking code formatting..."
cargo fmt --check

echo "✅ Code formatting is correct"

# Run clippy for linting
echo "🔍 Running clippy..."
cargo clippy

echo "✅ Clippy checks passed"

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "Next steps:"
echo "1. Update the repository URL in Cargo.toml"
echo "2. Update author information in Cargo.toml"
echo "3. Start implementing the core functionality"
echo "4. Add tests for each module"
echo ""
echo "Useful commands:"
echo "  cargo run -- --help                    # Show CLI help"
echo "  cargo watch -x run                     # Auto-reload on changes"
echo "  cargo test                             # Run tests"
echo "  cargo fmt                              # Format code"
echo "  cargo clippy                           # Run linter"
echo "  cargo audit                            # Security audit" 