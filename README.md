# Ordinator — Dotfiles and Environment Manager for macOS

[![CI](https://github.com/ordinators/ordinator/workflows/CI/badge.svg)](https://github.com/ordinators/ordinator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Ordinator** is a CLI tool written in Rust for managing macOS dotfiles, system settings, and secrets, allowing users to replicate their environment across machines in a secure, repeatable, and non-interactive way.

> **You've spent a lot of time customizing your setup—don't do it twice!** Ordinator helps you capture, version, and replicate your carefully crafted environment across any machine in minutes.

## Features

- **Profile-based dotfiles management** - Organize your dotfiles by environment (work, personal, laptop)
- **Secure secrets workflow** - Encrypt sensitive files with automatic plaintext prevention
- **Intuitive command structure** - `watch` to start tracking, `add` to update, `unwatch` to stop tracking
- **Homebrew package management** - Track and install packages per profile
- **Git integration** - Seamless commit, push, and sync operations
- **Automatic secrets scanning** - Detects potential plaintext secrets in tracked files
- **Bootstrap script generation** - Create setup scripts for new environments
- ...and more!

## Quick Start

### 1. Install Ordinator and Create Your First Dotfiles Repository

**Install Ordinator:**
```bash
brew install ordinators/ordinator/ordinator
```

**Initialize and set up your dotfiles:**
```bash
# Initialize a new repository with remote URL
ordinator init https://github.com/username/dotfiles.git

# Start watching your files
ordinator watch ~/.zshrc --profile work
ordinator watch ~/.gitconfig --profile work

# Set up age encryption for secure secrets
ordinator age setup --profile work

# Start watching sensitive files (secure workflow)
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets watch ~/.aws/credentials --profile work

# Export your Homebrew packages for reproducible environments
ordinator brew export --profile work

# Apply your configuration
ordinator apply --profile work

# Commit and push to GitHub
ordinator commit -m "Initial commit: track dotfiles with Ordinator"
ordinator push
```

### 2. Replicate Your Dotfiles Repository to Another Device

**Step 1:** Click the generated install link in your repository's README and run it in your terminal:
```bash
curl -fsSL https://raw.githubusercontent.com/username/dotfiles/master/scripts/install.sh | sh
```

**Step 2:** Apply your configuration:
```bash
ordinator apply --profile work
```

## How It Works

When you run `ordinator apply`, Ordinator:

1. **Generates the profile's bootstrap script** (if defined), which contains additional setup steps such as installing tools or configuring system settings.
2. **Decrypts and copies secrets** (if secrets management is configured and not skipped) - secrets are decrypted in memory and copied to target locations with secure permissions (600).
3. **Installs Homebrew packages** defined in the profile (if package management is configured and not skipped).
4. **Symlinks all tracked files** for the selected profile from your dotfiles repository into their correct locations in your home directory, backing up any existing files if configured.
5. **Performs safety checks** to avoid overwriting important files unless you use the `--force` flag.
6. **Supports dry-run mode** so you can preview all changes without making modifications by adding the `--dry-run` flag.





## Profiles

Ordinator supports multiple environment profiles (work, personal, laptop) that allow you to organize your dotfiles by context. Each profile contains its own set of tracked files, directories, and bootstrap scripts. When you apply a profile, only the files for that profile are symlinked, and files can be included in multiple profiles without conflicts. This enables you to maintain separate configurations for different environments while sharing common files across profiles.

## Secrets Management

Ordinator provides secure secrets management using Mozilla SOPS and age encryption. The workflow automatically encrypts sensitive files and stores only encrypted versions in your repository, eliminating the risk of accidentally committing plaintext secrets. The system includes automatic plaintext detection, profile-specific encryption keys, and key rotation capabilities for enhanced security.

### Security Features

- **Encrypted Storage**: All secrets are stored encrypted in the repository using SOPS and age
- **Secure Decryption**: During `apply`, secrets are decrypted in memory and copied to target locations with secure permissions (600)
- **No Plaintext in Repository**: Decrypted files are never stored in the repository
- **Temporary Processing**: Decryption happens in temporary files that are automatically cleaned up

> **Never commit your AGE key or other sensitive secrets to your repository.**
> The AGE key (typically at `~/.config/age/{profile}.key`) and SOPS configuration (typically at `~/.config/ordinator/.sops.{profile}.yaml`) should be kept secure and backed up separately.

## Uninstall and Restore

Ordinator provides a safe way to remove your dotfiles and restore your original configuration. The uninstall process removes all symlinks and optionally restores original files from backups, with interactive confirmations and dry-run mode to prevent accidental data loss.

## Documentation

- [Commands Reference](COMMANDS.md) - Complete CLI command documentation
- [Configuration Guide](CONFIGURATION.md) - Configuration file format and options
- [Examples Guide](EXAMPLES.md) - Common workflows and usage examples
- [Development Roadmap](DEVELOPMENT_ROADMAP.md) - Project development phases and progress

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Issues and Suggestions

Found a bug or have a feature request? We'd love to hear from you! Please visit our [Issues page](https://github.com/ordinators/ordinator/issues) to:

- Report bugs or unexpected behavior
- Suggest new features or improvements
- Ask questions about usage or configuration
- Share feedback on your experience with Ordinator

Your feedback helps make Ordinator better for everyone!