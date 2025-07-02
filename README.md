# Ordinator — Dotfiles and Environment Manager for macOS

[![CI](https://github.com/ordinators/ordinator/workflows/CI/badge.svg)](https://github.com/ordinators/ordinator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Ordinator** is a CLI tool written in Rust for managing macOS dotfiles, system settings, and secrets, allowing users to replicate their environment across machines in a secure, repeatable, and non-interactive way.

## Features

- ✅ **Dotfiles Management** - Track and sync dotfiles in Git with symlink management
- ✅ **Bootstrap Process** - Execute setup scripts and install tools non-interactively
- ✅ **Profile Support** - Environment profiles (work, personal, laptop)
- ✅ **Secrets Management** - Secure secrets using Mozilla SOPS + age encryption
- ✅ **Git Integration** - Git-inspired CLI commands without explicit git invocation
- ✅ **macOS-Specific** - System settings and Homebrew integration
- ✅ **Dry-Run Mode** - Preview changes before applying them

## Quick Start

```bash
# Install via Homebrew
brew install ordinators/ordinator/ordinator

# Initialize a new dotfiles repository
ordinator init --remote https://github.com/username/dotfiles

# Add your first dotfile
ordinator add ~/.zshrc

# Apply your configuration
ordinator apply
```

## Installation

### Homebrew (Recommended)
```bash
brew install ordinators/ordinator/ordinator
```

### Manual Installation
```bash
# Clone and build
git clone https://github.com/ordinators/ordinator.git
cd ordinator
cargo install --path .
```

## Documentation

- [Product Requirements Document](PRD.md) - Complete feature specification
- [Configuration Guide](CONFIGURATION.md) - Configuration file format and usage
- [Commands Reference](COMMANDS.md) - Complete CLI command documentation
- [Development Roadmap](DEVELOPMENT_ROADMAP.md) - Implementation plan
- [Test Plan](TEST_PLAN.md) - Testing strategy

## Development

```bash
# Setup development environment
./scripts/dev-setup.sh

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt
```

## License

MIT License - see [LICENSE](LICENSE) for details. 