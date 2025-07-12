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

# Update all tracked files for a profile
ordinator add --all --profile work
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

### Adding Encrypted Files

```bash
# Start watching an encrypted file
ordinator secrets watch ~/.ssh/config --profile work

# Update encrypted files with current content (secure workflow)
ordinator secrets add ~/.ssh/config --profile work

# Update all encrypted files for a profile
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

```bash
# 1. Start tracking a sensitive file
ordinator secrets watch ~/.ssh/config --profile work

# 2. Make changes to the original file
echo "Host new-server" >> ~/.ssh/config
echo "  HostName 192.168.1.100" >> ~/.ssh/config

# 3. Update the encrypted file (automatically encrypts)
ordinator secrets add ~/.ssh/config --profile work

# 4. Commit the encrypted file
ordinator commit -m "Update SSH configuration"

# 5. Push to remote
ordinator push
```

## Profile Management

### Creating Multiple Profiles

```bash
# Work profile
ordinator watch ~/.zshrc --profile work
ordinator watch ~/.gitconfig --profile work
ordinator secrets watch ~/.ssh/config --profile work

# Personal profile
ordinator watch ~/.zshrc --profile personal
ordinator watch ~/.config/alacritty --profile personal
ordinator secrets watch ~/.aws/credentials --profile personal

# Laptop profile
ordinator watch ~/.zshrc --profile laptop
ordinator watch ~/.config/karabiner --profile laptop
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

### Profile-Specific File Organization

```bash
# Files are organized by profile in the repository:
# files/work/.zshrc
# files/work/.gitconfig
# files/personal/.zshrc
# files/personal/.config/alacritty/
# files/laptop/.zshrc
# files/laptop/.config/karabiner/
```

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

# 4. Update tracked files
ordinator add ~/.zshrc --profile work

# 5. Commit and push
ordinator commit -m "Add new configuration"
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