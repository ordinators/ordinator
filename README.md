# Ordinator — Dotfiles and Environment Manager for macOS

[![CI](https://github.com/ordinators/ordinator/workflows/CI/badge.svg)](https://github.com/ordinators/ordinator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Ordinator** is a CLI tool written in Rust for managing macOS dotfiles, system settings, and secrets, allowing users to replicate their environment across machines in a secure, repeatable, and non-interactive way.

## Features

- **Profile-based dotfiles management** - Organize your dotfiles by environment (work, personal, laptop)
- **Profile-specific file storage** - Each profile has its own directory for file storage, preventing conflicts
- **Interactive profile selection** - Choose which profile to add files to with a simple prompt
- **Enhanced error handling** - Colorized output and clear guidance for resolving conflicts
- **Automatic secrets scanning** - Detects potential plaintext secrets in tracked files
- **SOPS and age integration** - Encrypt sensitive files with industry-standard tools
- **Homebrew package management** - Track and install packages per profile
- **Bootstrap script generation** - Create setup scripts for new environments
- **Git integration** - Seamless commit, push, and sync operations
- **Auto-generated README** - Professional documentation with installation instructions

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

# Add your first file
ordinator add ~/.zshrc --profile work

# Set up secrets management for sensitive files
ordinator secrets setup --profile work

# Add a sensitive file securely (encrypts before storing)
# TODO: This should be: ordinator secrets add ~/.ssh/config --profile work
# For now, we need to manually handle the encryption workflow:
ordinator add ~/.ssh/config --profile work
ordinator secrets encrypt files/work/.ssh/config
rm files/work/.ssh/config  # Remove plaintext version

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

## Phase 4.5 Features

### Profile-Specific File Storage

Ordinator now stores files in profile-specific directories, preventing conflicts between different environments:

```bash
# Add files to different profiles
ordinator add ~/.zshrc --profile work
ordinator add ~/.zshrc --profile personal
# Creates: files/work/.zshrc and files/personal/.zshrc

# Apply specific profile
ordinator apply --profile work
# Symlinks files/work/.zshrc to ~/.zshrc
```

### Interactive Profile Selection

When adding files without specifying a profile, Ordinator prompts you to choose:

```bash
ordinator add ~/.gitconfig
# Prompts:
# Select a profile to add this file to:
#   1. default
#   2. work  
#   3. personal
# Enter number (default: default):
```

### Enhanced Error Handling

- **Colorized output** for better readability
- **Conflict detection** when the same file exists in multiple profiles
- **Clear guidance** for resolving issues
- **Progress indicators** for file operations

### Backward Compatibility

Existing repositories with flat `files/` structure continue to work seamlessly. Ordinator automatically detects and uses the appropriate file structure.

When you run `ordinator apply`, Ordinator:

1. **Generates the profile's bootstrap script** (if defined), which contains additional setup steps such as installing tools or configuring system settings.
2. **Decrypts secrets** (if secrets management is configured and not skipped), making encrypted files available for use.
3. **Installs Homebrew packages** defined in the profile (if package management is configured and not skipped).
4. **Symlinks all tracked files** for the selected profile from your dotfiles repository into their correct locations in your home directory, backing up any existing files if configured.
5. **Performs safety checks** to avoid overwriting important files unless you use the `--force` flag.
6. **Supports dry-run mode** so you can preview all changes without making modifications by adding the `--dry-run` flag.

**Order of Operations:**
The apply command follows a specific order to ensure dependencies are satisfied:
- Homebrew packages are installed **before** symlinks are created to prevent broken symlinks to Homebrew-installed tools
- Secrets are decrypted **before** symlinks to ensure encrypted files are available
- Bootstrap scripts are generated **first** for user review and manual execution

This makes it easy to replicate your environment on any machine in a safe, repeatable, and automated way.

## Bootstrap

Ordinator generates a profile-specific bootstrap script (e.g., `scripts/bootstrap-work.sh`) after applying your profile. This script automates extra setup steps (like installing packages or configuring system settings) and may require elevated privileges.

**You must review and run the script manually** for safety:
```bash
chmod +x scripts/bootstrap-work.sh
./scripts/bootstrap-work.sh
```
Manual execution ensures you control any privileged or system-altering commands.

## Profiles

Profiles in Ordinator are independent sets of tracked files, directories, and bootstrap scripts. When you apply a profile, only the files listed in that profile are symlinked; files can be included in multiple profiles if desired. Applying a new profile does not remove files from previous profiles—those files remain unless you manually clean them up or they are overwritten by the new profile.

Ordinator supports multiple environment profiles (e.g., work, personal, laptop). Each profile can have its own set of tracked files, directories, and bootstrap script.

- List available profiles:
  ```bash
  ordinator profiles
  ```
- Apply a specific profile:
  ```bash
  ordinator apply --profile work
  ```

## Secrets Management

Ordinator integrates with Mozilla SOPS and age for secure secrets management. This enables you to encrypt your secrets and safely commit them to your GitHub repository without fear of compromise—only those with your private AGE key can decrypt them.

Ordinator also scans tracked files for potential plaintext secrets and warns you if any are detected, helping prevent accidental exposure of sensitive information.

- Encrypt a file:
  ```bash
  ordinator secrets encrypt secrets.yaml
  ```
- Decrypt a file:
  ```bash
  ordinator secrets decrypt secrets.enc.yaml
  ```

> **Never commit your AGE key or other sensitive secrets to your repository.**
> The AGE key (typically at `~/.config/ordinator/age/key.txt`) and SOPS configuration (typically at `