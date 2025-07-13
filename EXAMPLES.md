# Ordinator Examples

This document provides comprehensive examples of common Ordinator workflows and use cases.

## Table of Contents

- [Quick Start](#quick-start)
- [Repository Management](#repository-management)
- [File Management](#file-management)
- [Secrets Management](#secrets-management)
- [Profile Management](#profile-management)
- [Bootstrap Scripts](#bootstrap-scripts)
- [Package Management](#package-management)
- [Advanced Workflows](#advanced-workflows)

## Quick Start

### Initialize a New Dotfiles Repository

```bash
# Create a new dotfiles repository
ordinator init https://github.com/username/dotfiles.git

# Or initialize with a specific directory
ordinator init https://github.com/username/dotfiles.git ~/.dotfiles
```

### Set Up Your First Profile

```bash
# Start watching regular files
ordinator watch ~/.zshrc --profile work
ordinator watch ~/.gitconfig --profile work
ordinator watch ~/.config/nvim --profile work

# Start watching encrypted files
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets watch ~/.aws/credentials --profile work

# Apply the profile to your system
ordinator apply --profile work
```

## Repository Management

### Initialize with Existing Repository

```bash
# Clone and set up an existing dotfiles repository
ordinator init https://github.com/username/dotfiles.git

# The repository will be cloned and configured automatically
# You can then apply profiles or add new files
```

### Git Operations

```bash
# Check repository status
ordinator status

# Commit changes
ordinator commit -m "Add new configuration files"

# Push to remote
ordinator push

# Pull latest changes
ordinator pull
```

## File Management

### Adding Regular Files

```bash
# Start watching a file (initial tracking)
ordinator watch ~/.zshrc --profile work

# Update tracked files with current content
ordinator add ~/.zshrc --profile work

# Update all tracked files for a profile (bulk operation)
ordinator add --all --profile work
```

### Bulk Operations

Both `add` and `secrets add` support bulk operations with the `--all` flag:

```bash
# Update all regular files for a profile
ordinator add --all --profile work

# Update all encrypted files for a profile
ordinator secrets add --all --profile work

# No manual file path specification required
# Commands loop through files/secrets arrays automatically
```

### Managing Multiple Profiles

```bash
# Start watching the same file in different profiles
ordinator watch ~/.zshrc --profile work
ordinator watch ~/.zshrc --profile personal

# Each profile gets its own copy in files/<profile>/
# files/work/.zshrc and files/personal/.zshrc
```

### Removing Files from Tracking

```bash
# Stop watching a file
ordinator unwatch ~/.zshrc --profile work

# The file will be removed from tracking but not deleted from disk
```

## Secrets Management

> **Security Note:** Decrypted secrets are never stored in the repository. They are only copied to their target locations during `ordinator apply` and are written with secure permissions (0600).

### Adding Encrypted Files

```bash
# Start watching an encrypted file
ordinator secrets watch ~/.ssh/config --profile work

# Update encrypted files with current content (secure workflow)
ordinator secrets add ~/.ssh/config --profile work

# Update all encrypted files for a profile (bulk operation)
ordinator secrets add --all --profile work
```

### Managing Encrypted Files

```bash
# List encrypted files
ordinator secrets list

# Stop watching an encrypted file
ordinator secrets unwatch ~/.ssh/config --profile work

# Scan for plaintext secrets
ordinator secrets scan
```

### Secure Workflow Example

> **Security Note:** Decrypted secrets are never stored in the repository. They are only present in memory or at their destination after `ordinator apply`.

```bash
# 1. Start tracking sensitive files
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets watch ~/.aws/credentials --profile work

# 2. Make changes to the original files
echo "Host new-server" >> ~/.ssh/config
echo "  HostName 192.168.1.100" >> ~/.ssh/config

# 3. Bulk update all encrypted files (automatically encrypts)
ordinator secrets add --all --profile work

# 4. Commit the encrypted files
ordinator commit -m "Update encrypted configuration files"

# 5. Push to remote
ordinator push
```

### Secrets Array Management

The `secrets` array in your configuration tracks direct paths to source files:

```toml
[profiles.work]
files = ["~/.gitconfig"]
secrets = ["~/.ssh/config", "~/.aws/credentials", "~/.config/api_keys.json"]
enabled = true
description = "Work environment with encrypted secrets"
```

**Bulk Operations:**
- `ordinator secrets add --all` loops through all files in the `secrets` array
- Each source file is read directly, re-encrypted, and updated in the repository
- No manual file path specification required for bulk operations

## Profile Management

### Creating Multiple Profiles

```bash
# Work profile
ordinator watch ~/.zshrc --profile work
ordinator watch ~/.gitconfig --profile work
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets watch ~/.aws/credentials --profile work

# Personal profile
ordinator watch ~/.zshrc --profile personal
ordinator watch ~/.config/alacritty --profile personal
ordinator secrets watch ~/.aws/credentials --profile personal

# Laptop profile
ordinator watch ~/.zshrc --profile laptop
ordinator watch ~/.config/karabiner --profile laptop
```

### Profile Configuration Structure

Your `ordinator.toml` will contain separate arrays for different file types:

```toml
[profiles.work]
files = ["~/.zshrc", "~/.gitconfig"]
secrets = ["~/.ssh/config", "~/.aws/credentials"]
directories = ["~/.config/nvim"]
enabled = true
description = "Work environment with encrypted secrets"

[profiles.personal]
files = ["~/.zshrc"]
secrets = ["~/.config/api_keys.json"]
directories = ["~/.config/alacritty"]
enabled = true
description = "Personal environment configuration"
```

### Applying Profiles

```bash
# Apply a specific profile
ordinator apply --profile work

# Apply all profiles
ordinator apply --all

# Apply with bootstrap script
ordinator apply --profile work --bootstrap

# Skip bootstrap script
ordinator apply --profile work --skip-bootstrap
```

### Secrets Decryption During Apply

When you run `ordinator apply`, secrets are securely decrypted and copied to their target locations:

```bash
# Before apply - encrypted files in repository
ls ~/.dotfiles/files/work/
# .ssh/config (encrypted)
# .aws/credentials (encrypted)

# Run apply to decrypt and copy secrets
ordinator apply --profile work

# After apply - decrypted files at target locations
ls ~/.ssh/
# config (decrypted, readable by SSH)

ls ~/.aws/
# credentials (decrypted, readable by AWS CLI)

# Repository still contains only encrypted files
ls ~/.dotfiles/files/work/
# .ssh/config (encrypted)
# .aws/credentials (encrypted)
```

**Security Note:** Decrypted secrets are never stored in the repository. They are only present in memory during decryption and at their destination after `ordinator apply`.

### Profile-Specific File Organization

```bash
# Files are organized by profile in the repository:
# files/work/.zshrc
# files/work/.gitconfig
# files/work/.ssh/config (encrypted)
# files/work/.aws/credentials (encrypted)
# files/personal/.zshrc
# files/personal/.config/alacritty/
# files/personal/.config/api_keys.json (encrypted)
# files/laptop/.zshrc
# files/laptop/.config/karabiner/
```

**File Types:**
- **Regular files**: Stored in `files/<profile>/` (managed by `files` array)
- **Encrypted files**: Stored encrypted in `files/<profile>/` (managed by `secrets` array)
- **Directories**: Stored in `files/<profile>/` (managed by `directories` array)

## Bootstrap Scripts

### Generating Bootstrap Scripts

```bash
# Generate bootstrap script for a profile
ordinator bootstrap --profile work

# Edit bootstrap script before applying
ordinator bootstrap --profile work --edit

# The script will be generated and you'll be shown how to run it
# Example output:
# Bootstrap script generated: /path/to/scripts/bootstrap-work.sh
# Run: bash /path/to/scripts/bootstrap-work.sh
```

### Manual Bootstrap Execution

```bash
# After generating the script, run it manually
bash ~/.dotfiles/scripts/bootstrap-work.sh

# The script will install packages and set up your environment
```

## Package Management

### Exporting Homebrew Packages

```bash
# Export currently installed packages for a profile
ordinator brew export --profile work

# This adds packages to your configuration
# They will be installed when you apply the profile
```

### Installing Packages

```bash
# Install packages for a profile
ordinator brew install --profile work

# Or install during apply
ordinator apply --profile work

# Skip package installation
ordinator apply --profile work --skip-brew
```

### Managing Package Lists

```bash
# List packages for a profile
ordinator brew list --profile work

# Export packages for multiple profiles
ordinator brew export --profile work
ordinator brew export --profile personal
ordinator brew export --profile laptop
```

## Advanced Workflows

### Setting Up a New Machine

```bash
# 1. Clone your dotfiles repository
git clone https://github.com/username/dotfiles.git ~/.dotfiles
cd ~/.dotfiles

# 2. Initialize Ordinator
ordinator init

# 3. Apply your work profile
ordinator apply --profile work --bootstrap

# 4. Run the bootstrap script
bash ~/.dotfiles/scripts/bootstrap-work.sh
```

### Collaborative Development

```bash
# 1. Pull latest changes
ordinator pull

# 2. Apply updates
ordinator apply --profile work

# 3. Make your changes
echo "new_config" >> ~/.zshrc
echo "Host new-server" >> ~/.ssh/config

# 4. Update all tracked files (bulk operation)
ordinator add --all --profile work
ordinator secrets add --all --profile work

# 5. Commit and push
ordinator commit -m "Update configuration files"
ordinator push
```

### Managing Multiple Environments

```bash
# Work environment
ordinator apply --profile work
# Installs work-specific packages and configurations

# Switch to personal environment
ordinator apply --profile personal
# Installs personal packages and configurations

# Laptop environment
ordinator apply --profile laptop
# Installs laptop-specific packages and configurations
```

### Security Best Practices

```bash
# 1. Always use secrets commands for sensitive files
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets watch ~/.aws/credentials --profile work

# 2. Never add plaintext secrets
# ❌ ordinator watch ~/.ssh/config --profile work
# ✅ ordinator secrets watch ~/.ssh/config --profile work

# 3. Scan for plaintext secrets before committing
ordinator secrets scan

# 4. Use force flag only when necessary
ordinator commit -m "Update config" --force
```

### Age Encryption Utilities

```bash
# Manual encryption for files not in repository
ordinator age encrypt ~/temp-secret.yaml
ordinator age decrypt ~/temp-secret.enc.yaml

# Set up age encryption for a profile
ordinator age setup --profile work
ordinator age validate --profile work

# Rotate encryption keys
ordinator age rotate-keys --profile work
ordinator age rotate-keys --profile work --backup-old-key

# Preview key rotation
ordinator age rotate-keys --profile work --dry-run
```

### Troubleshooting

```bash
# Check repository status
ordinator status

# Repair broken symlinks
ordinator repair

# Validate configuration
ordinator validate-config

# Check for issues
ordinator secrets scan
```

This examples guide covers the most common workflows and use cases for Ordinator. For more detailed information, see the main documentation and command reference. 