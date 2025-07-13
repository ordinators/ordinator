# Ordinator Configuration Guide

This document describes the configuration system for Ordinator, including the structure and options available in `ordinator.toml`.

---

## Overview

Ordinator uses a single TOML configuration file, typically located at `~/.dotfiles/ordinator.toml`, to manage dotfiles, profiles, secrets, and global settings. This file is created automatically when you run `ordinator init`.

---

## Configuration File Location

- **Default location:** `~/.dotfiles/ordinator.toml`
- Ordinator will also look for `ordinator.toml` in the current working directory.

---

## Example `ordinator.toml`

```toml
[global]
default_profile = "default"
auto_push = false
create_backups = true
exclude = ["*.bak"]

[profiles.default]
files = ["~/.zshrc", "~/.gitconfig"]
directories = ["~/.config/nvim"]
secrets = ["~/.ssh/config"]
homebrew_packages = ["git", "neovim", "ripgrep"]
enabled = true
description = "Default profile for basic dotfiles"
exclude = ["*.bak"]

[profiles.work]
files = ["~/.gitconfig"]
directories = []
secrets = ["~/.ssh/config", "~/.aws/credentials"]
homebrew_packages = ["git", "neovim", "ripgrep", "sops", "age", "jq", "yq"]
enabled = true
description = "Work environment profile"

[profiles.personal]
files = ["~/.zshrc"]
directories = []
secrets = ["~/.config/api_keys.json"]
homebrew_packages = ["git", "neovim"]
enabled = true
description = "Personal environment profile"

[secrets]
age_key_file = "~/.config/ordinator/age/key.txt"
sops_config = "~/.sops.yaml"
encrypt_patterns = ["secrets/*.yaml"]
exclude_patterns = ["*.bak"]

[readme]
auto_update = false
update_on_changes = ["profiles", "bootstrap"]
```

---

## Sections

### `[global]`
- `default_profile` (string): The profile to use by default.
- `auto_push` (bool): If true, automatically push changes after successful operations.
- `create_backups` (bool): If true, create backups before making changes.
- `exclude` (array of strings): Glob patterns for files or directories to exclude from tracking or symlinking (applies globally).

### `[profiles.<name>]`
- `files` (array of strings): List of files tracked by this profile.
  - Managed by `ordinator watch` and `ordinator unwatch` commands
  - Files are stored in `files/<profile>/` subdirectories
  - Updated by `ordinator add` command
- `secrets` (array of strings): List of encrypted files tracked by this profile.
  - Managed by `ordinator secrets watch` and `ordinator secrets unwatch` commands
  - Files are stored encrypted in `files/<profile>/` subdirectories
  - Updated by `ordinator secrets add` command (secure workflow)
  - Contains direct paths to source files (e.g., `~/.ssh/config`)
  - Used by `ordinator secrets add --all` for bulk re-encryption
- `directories` (array of strings): List of directories tracked by this profile.
  - Managed by `ordinator watch` and `ordinator unwatch` commands
  - Directories are stored in `files/<profile>/` subdirectories
  - Updated by `ordinator add` command
- `homebrew_packages` (array of strings, optional): List of Homebrew packages to install for this profile.
  - Can include both formulae and casks
  - Packages are installed using `brew install` for formulae and `brew install --cask` for casks
  - Installed automatically when running `ordinator apply` (unless `--skip-brew` is used)
  - Can be exported from current system using `ordinator brew export --profile <name>`
  - Example: `["git", "neovim", "ripgrep", "sops", "age"]`
- `bootstrap_script` (string, optional): Path to a bootstrap script for this profile.
  - Relative path from the dotfiles directory (e.g., "scripts/bootstrap-default.sh")
  - Absolute paths are also supported (e.g., "/path/to/script.sh")
  - If not specified, no bootstrap script will be generated for this profile
  - The script will be generated automatically when you run `ordinator apply`
  - You can edit the script using `ordinator bootstrap --edit --profile <name>`
- `enabled` (bool): Whether this profile is active.
- `description` (string, optional): Description of the profile.
- `exclude` (array of strings): Glob patterns for files or directories to exclude for this profile (overrides or adds to global exclusions).

## Bootstrap Scripts

Ordinator supports profile-specific bootstrap scripts that help automate environment setup on new machines.

### Bootstrap Script Features

**Automatic Generation:**
- Bootstrap scripts are generated automatically when you run `ordinator apply`
- Scripts are created in the dotfiles repository (typically under `scripts/`)
- Scripts are never executed automatically by Ordinator (security feature)

**Safety Validation:**
- Ordinator validates bootstrap scripts for safety before presenting them
- Safety levels: Safe, Warning, Dangerous, Blocked
- Blocked scripts contain extremely dangerous commands (e.g., `rm -rf /`)
- Dangerous scripts contain commands like `sudo` (requires manual review)
- Warning scripts contain potentially risky commands (e.g., `rm -rf <path>`)

**Editing Workflow:**
1. Run `ordinator apply` to generate the bootstrap script
2. Run `ordinator bootstrap --edit --profile <name>` to edit the script
3. Add your setup commands (e.g., `brew install git`, `defaults write ...`)
4. Commit and push your changes
5. On new machines, run the script manually as instructed

### Example Bootstrap Script

```bash
#!/bin/bash
# Bootstrap script for profile: work
# Generated by Ordinator
# 
# This script will be executed to set up your environment.
# Edit this file to customize your bootstrap process.

set -e  # Exit on any error

echo "Starting bootstrap for profile: work"

# Add your bootstrap commands here
# Examples:
# brew install git neovim ripgrep
# defaults write com.apple.dock autohide -bool true
# npm install -g typescript

echo "Bootstrap complete for profile: work"
```

### Configuration Example

```toml
[profiles.work]
files = ["~/.zshrc", "~/.gitconfig"]
bootstrap_script = "scripts/bootstrap-work.sh"
enabled = true
description = "Work environment with development tools"

[profiles.laptop]
files = ["~/.zshrc"]
bootstrap_script = "scripts/bootstrap-laptop.sh"
enabled = true
description = "Laptop setup with minimal tools"
```

### `[secrets]`
- `age_key_file` (string, optional): Path to the age key file for decryption.
  - Must contain a valid age key in the format `age1...`
  - Used for both encryption and decryption operations
  - Default location: `~/.config/ordinator/age/key.txt` (or `{profile}.txt` for profile-specific keys)
  - If not specified, SOPS will use default key locations
  - **Security Note:** Decrypted secrets are never stored in the repository. They are only present in memory or at their destination after `ordinator apply`.

- `sops_config` (string, optional): Path to the SOPS configuration file.
  - Used to configure SOPS encryption settings
  - Supports multiple encryption methods (age, GPG, KMS)
  - If not specified, SOPS will use default configuration

- `encrypt_patterns` (array of strings): Glob patterns for files to encrypt.
  - Supports standard glob patterns (e.g., `*.yaml`, `secrets/**/*`)
  - Files matching these patterns will be automatically encrypted
  - Can be overridden by `exclude_patterns`
  - Example patterns:
    - `secrets/*.yaml` - Encrypt all YAML files in secrets directory
    - `*.key` - Encrypt all key files
    - `secrets/**/*` - Encrypt all files in secrets directory recursively

- `exclude_patterns` (array of strings): Glob patterns for files to exclude from encryption.
  - Supports standard glob patterns
  - Files matching these patterns will never be encrypted, even if they match `encrypt_patterns`
  - Useful for excluding backup files or already encrypted files
  - Example patterns:
    - `*.bak` - Exclude backup files
    - `**/*.enc.yaml` - Exclude already encrypted YAML files
    - `secrets/excluded/**/*` - Exclude specific directory from encryption

- `encryption_format` (string, optional): Format for encrypted files.
  - Default: `"{stem}.enc.{ext}"` for YAML files, `"{stem}.enc"` for others
  - Supports template variables:
    - `{stem}` - File stem (name without extension)
    - `{ext}` - File extension
    - `{file}` - Full file name
  - Example: `"{stem}.sops.{ext}"` to use .sops extension

- `encryption_method` (string, optional): Preferred encryption method.
  - Default: "age"
  - Supported values: "age", "gpg", "kms"
  - Must match available encryption keys in SOPS configuration

### `[age]`
- `key_file` (string, optional): Path to the age key file.
  - Profile-specific keys stored as `~/.config/age/{profile}.key`
  - Used for encryption and decryption operations
  - Generated automatically by `ordinator age setup`

- `sops_config` (string, optional): Path to the SOPS configuration file.
  - Profile-specific configs stored as `~/.config/ordinator/.sops.{profile}.yaml`
  - Configures age encryption method
  - Generated automatically by `ordinator age setup`

- `rotation_backup` (bool, optional): Whether to backup old keys during rotation.
  - Default: `false`
  - When `true`, old keys are preserved as `.backup` files
  - Useful for recovery during key rotation

- `auto_validate` (bool, optional): Whether to validate setup after operations.
  - Default: `true`
  - When `true`, automatically validates age setup after key generation
  - Ensures encryption/decryption works correctly

### `[readme]`
- `auto_update` (bool): Whether to automatically update README.md when configuration changes.
  - Default: `false` (manual mode)
  - When `true`, README is automatically regenerated when profiles, bootstrap, or AGE key info changes
  - When `false`, users get notifications about potential README updates

- `update_on_changes` (array of strings): Specific changes that trigger README updates.
  - Default: `["profiles", "bootstrap"]`
  - Supported values: `"profiles"`, `"bootstrap"`, `"age_key"`
  - Only relevant when `auto_update = true`
  - Controls which configuration changes trigger automatic README updates

**README Features:**
- **Interactive copy buttons** for easy command copying
- **Private repository support** with PAT input form
- **Automatic repository URL detection** from Git remote
- **Install script generation** in `scripts/install.sh` (in the dotfiles repository)
- **README generation** in `README.md` (root of the dotfiles repository)
- **State tracking** in `readme_state.json` (root of the dotfiles repository)
- **Comprehensive sections** including quick install, profiles, AGE setup, troubleshooting
- **Security notes** and best practices
- **Warning system** for missing remote configuration
- **Git integration** - All generated files are committed to the repository

**Configuration Example:**
```toml
[readme]
auto_update = true
update_on_changes = ["profiles", "bootstrap", "age_key"]
```

**Auto-Update Behavior:**
- When `auto_update = true`, README is automatically updated when:
  - Profile configurations change (if "profiles" is in `update_on_changes`)
  - Bootstrap scripts change (if "bootstrap" is in `update_on_changes`)
  - AGE key configuration changes (if "age_key" is in `update_on_changes`)
- When `auto_update = false`, users receive warnings about outdated READMEs
- Manual updates can be triggered with `ordinator readme default` or `ordinator readme preview`

---

## Homebrew Package Management

Ordinator supports managing Homebrew packages per profile, allowing you to maintain reproducible development environments across different machines.

### Homebrew Package Configuration

**Profile-based Packages:**
- Each profile can specify a `homebrew_packages` array
- Packages are installed automatically during `ordinator apply`
- Supports both formulae and casks
- Packages are installed before symlinks are created to prevent broken links

**Export Process:**
When you run `ordinator brew export --profile <name>`, the following happens:
1. **Package Detection**: Ordinator detects currently installed Homebrew formulae and casks
2. **Configuration Update**: Updates the profile's `homebrew_packages` array in `ordinator.toml`
3. **Version Preservation**: Captures package versions for reproducible environments
4. **Profile Association**: Associates packages with the specified profile

**Installation Process:**
When you run `ordinator apply` or `ordinator brew install`, the following happens:
1. **Package Resolution**: Ordinator reads the profile's `homebrew_packages` array
2. **Installation**: Uses `brew install` for formulae and `brew install --cask` for casks
3. **Error Handling**: Continues installation even if some packages fail
4. **Progress Feedback**: Provides installation status and progress information

### Example Homebrew Workflow

```bash
# 1. Export current Homebrew packages to work profile
ordinator brew export --profile work

# 2. Review what packages will be installed
ordinator brew list --profile work

# 3. Apply configuration (includes Homebrew installation)
ordinator apply --profile work

# 4. Or install packages separately
ordinator brew install --profile work
```

### Homebrew Package Best Practices

1. **Profile Organization**
   - Use different packages for different environments (work, personal, laptop)
   - Keep packages minimal and focused per profile
   - Document why specific packages are needed

2. **Version Management**
   - Export packages when setting up new environments
   - Review package lists regularly for updates
   - Consider pinning specific versions for critical tools

3. **Installation Order**
   - Homebrew packages are installed before symlinks
   - This prevents broken symlinks to Homebrew-installed tools
   - Use `--skip-brew` to skip package installation during apply

4. **Error Handling**
   - Missing packages don't block the entire installation
   - Review installation logs for failed packages
   - Re-run installation for failed packages if needed

## Secrets Array Management

The `secrets` array in each profile configuration contains direct paths to source files that should be encrypted and tracked. This array is used for bulk operations and secure workflow management.

> **Security Note:** Decrypted secrets are never stored in the repository. They are only present in memory or at their destination after `ordinator apply`.

### Secrets Array Features

**Direct Source Paths:**
- Contains absolute paths to source files (e.g., `~/.ssh/config`, `~/.aws/credentials`)
- Used by `ordinator secrets add --all` for bulk re-encryption
- Automatically managed by `ordinator secrets watch` and `ordinator secrets unwatch` commands

**Bulk Operations:**
- `ordinator secrets add --all` loops through all files in the `secrets` array
- Each source file is read directly, re-encrypted, and the encrypted version is updated in the repository
- No manual file path specification required for bulk operations

**Secure Workflow:**
```bash
# 1. Add files to secrets tracking
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets watch ~/.aws/credentials --profile work

# 2. Make changes to source files
echo "Host new-server" >> ~/.ssh/config

# 3. Bulk update all encrypted files
ordinator secrets add --all --profile work

# 4. Commit and push changes
ordinator commit -m "Update encrypted configuration files"
ordinator push
```

**Configuration Example:**
```toml
[profiles.work]
files = ["~/.gitconfig"]
secrets = ["~/.ssh/config", "~/.aws/credentials", "~/.config/api_keys.json"]
enabled = true
description = "Work environment with encrypted secrets"
```

## SOPS Setup Process

When you run `ordinator secrets setup`, the following happens:

1. **Installation Check**: Ordinator checks if SOPS and age are installed
   - If missing, installs them via Homebrew
   - Shows installation paths if found

2. **Age Key Generation**: Creates an age encryption key for the profile
   - Location: `~/.config/age/{profile}.key`
   - Format: Age v2 private key
   - Used for both encryption and decryption

3. **SOPS Configuration**: Creates `.sops.yaml` configuration file
   - Location: `~/.config/ordinator/.sops.{profile}.yaml`
   - Configures age encryption method
   - Sets up creation rules for encrypted files

4. **Configuration Update**: Updates `ordinator.toml` with secrets settings
   - Adds `age_key_file` path
   - Adds `sops_config` path
   - Sets up default encryption patterns
   - Configures exclusion patterns

### Example Setup Output

```bash
$ ordinator secrets setup --profile work
✅ SOPS and age are already installed
✅ Age key already exists: ~/.config/age/work.key
✅ SOPS config already exists: ~/.config/ordinator/.sops.work.yaml
✅ SOPS and age setup complete for profile: work
   Age key: ~/.config/age/work.key
   SOPS config: ~/.config/ordinator/.sops.work.yaml
```

### Generated Configuration

After setup, your `ordinator.toml` will include:

```toml
[secrets]
age_key_file = "~/.config/ordinator/age/work.key"
sops_config = "~/.config/ordinator/.sops.work.yaml"
encrypt_patterns = [
    "secrets/**/*.yaml",
    "secrets/**/*.json",
    "*.key"
]
exclude_patterns = [
    "*.bak",
    "**/*.enc.yaml",
    "secrets/excluded/**/*"
]
```

---

## Customizing Your Configuration

- Add or remove profiles by editing the `[profiles.<name>]` sections.
- Add files or directories to profiles to track them with Ordinator.
- Use the `[secrets]` section to specify which files should be encrypted and how secrets are managed.
- Use the `[global]` section to set defaults and enable/disable features.
- Use the `exclude` field in `[global]` or `[profiles.<name>]` to prevent certain files or directories from being tracked or symlinked. Profile-level `exclude` patterns take precedence over global ones.
- Configure encryption patterns and exclusions to control which files are automatically encrypted.
- Customize encryption format and method to match your security requirements.

## Secrets Scanning

Ordinator automatically scans for plaintext secrets to help prevent accidental exposure of sensitive information.

### Automatic Scanning

**Add Command:**
- Automatically scans files when adding them to tracking
- Warns about detected secrets but doesn't block the operation
- Provides suggestions for encrypting detected secrets

**Commit Command:**
- Scans all tracked files before committing
- Blocks commit with error code 1 if secrets are found
- Use `--force` flag to skip scanning and commit anyway

### Manual Scanning

Use `ordinator secrets scan` to manually check for secrets:
- Scans all tracked files across all profiles
- Uses advanced regex-based heuristics to detect common secret patterns
- **Detects secrets with prefixes** (e.g., `test_api_key`, `prod_password`)
- **Handles special characters** in filenames and secret values
- **Supports Unicode filenames** and international character sets
- Lists secret types found without exposing actual values
- Always exits with error code 1 if secrets are detected
- **Robust error handling** for permission issues, binary files, and large files

### Detected Secret Types

The scanner looks for:
- Passwords and API keys (including prefixed variants)
- Access tokens and credentials
- Private keys and certificates
- Database connection strings
- Configuration secrets
- **Special characters and Unicode content**
- **Binary files and large files** (handled gracefully)

## Encryption Best Practices

1. **Key Management**
   - Store age keys securely
   - Use separate keys for different environments (work/personal)
   - Backup encryption keys in a secure location

2. **Pattern Configuration**
   - Encrypt sensitive files by default
   - Use exclude patterns for backup files
   - Keep encryption patterns consistent across profiles

3. **File Organization**
   - Store encrypted files in dedicated directories
   - Use consistent naming conventions
   - Document encryption requirements in README files

4. **Security**
   - Never commit encryption keys to version control
   - Use strong encryption methods
   - Regularly rotate encryption keys

5. **Configuration Example**
   ```toml
   [secrets]
   age_key_file = "~/.config/ordinator/age/work.key"
   sops_config = "~/.config/ordinator/.sops.work-config.yaml"
   encrypt_patterns = [
     "secrets/**/*.yaml",
     "secrets/**/*.json",
     "*.key"
   ]
   exclude_patterns = [
     "*.bak",
     "**/*.enc.yaml",
     "secrets/excluded/**/*"
   ]
   encryption_format = "{stem}.sops.{ext}"
   encryption_method = "age"
   ```

## Security Best Practices

### Key Management
- **Separate keys per environment**: Use different age keys for work, personal, and laptop profiles
- **Secure key storage**: Store age keys in `~/.config/ordinator/age/` with restricted permissions (600)
- **Key backup**: Backup age keys securely (not in version control)
- **Key rotation**: Regularly rotate encryption keys for sensitive data

### File Organization
- **Dedicated secrets directories**: Store sensitive files in `secrets/` directories
- **Consistent naming**: Use clear naming conventions for encrypted files
- **Documentation**: Document which files contain secrets and why
- **Exclusion patterns**: Exclude backup files and already encrypted files

### Configuration Security
- **Profile isolation**: Use separate profiles for different security contexts
- **Pattern validation**: Regularly review encryption patterns for completeness
- **Access control**: Limit access to SOPS configuration files
- **Audit trails**: Use `ordinator secrets list` to audit encrypted files

### Operational Security
- **Dry-run testing**: Always test encryption/decryption with `--dry-run` first
- **Backup before encryption**: Ensure original files are backed up before encryption
- **Verification**: Verify decryption works before removing original files
- **Monitoring**: Monitor for unexpected encryption/decryption events

### Example Secure Workflow
```bash
# 1. Set up secrets management for work profile
ordinator secrets setup --profile work

# 2. Add sensitive files to tracking (automatically scans for secrets)
ordinator add ~/.ssh/config --profile work
ordinator add ~/.config/api_keys.json --profile work

# 3. Scan for any remaining secrets
ordinator secrets scan --profile work

# 4. Encrypt sensitive files
ordinator secrets encrypt ~/.ssh/config
ordinator secrets encrypt ~/.config/api_keys.json

# 5. Verify encryption worked
ordinator secrets list

# 6. Commit changes (automatically scans for secrets)
ordinator commit -m "Add encrypted configuration files"

# 7. Apply configuration with decryption
ordinator apply --profile work

# 8. Verify decryption worked
ls -la ~/.ssh/config
```

---

## Repository Structure

When you initialize an Ordinator dotfiles repository, it creates the following structure:

```
dotfiles-repo/
├── .git/                   # Git repository
├── .gitignore             # Security-focused ignore rules
├── ordinator.toml         # Configuration file
├── README.md              # Auto-generated documentation
├── files/                 # Profile-specific file storage
│   ├── default/           # Files for default profile
│   │   ├── .zshrc
│   │   ├── .gitconfig
│   │   └── .ssh/config    # Encrypted file
│   ├── work/              # Files for work profile
│   │   ├── .gitconfig
│   │   ├── .ssh/config    # Encrypted file
│   │   └── .aws/credentials # Encrypted file
│   └── personal/          # Files for personal profile
│       ├── .zshrc
│       ├── .config/alacritty/alacritty.yml
│       └── .config/api_keys.json # Encrypted file
└── scripts/               # Bootstrap scripts
    ├── bootstrap-default.sh
    ├── bootstrap-work.sh
    └── bootstrap-personal.sh
```

### Profile-Specific File Storage

**New Structure (Phase 4.5+):**
- Files are stored in `files/<profile>/` subdirectories
- Each profile has its own directory for file storage
- Same file can exist in multiple profiles with different content
- Backward compatibility with flat `files/` structure

**File Types:**
- **Regular files**: Stored in `files/<profile>/` (managed by `files` array)
- **Encrypted files**: Stored encrypted in `files/<profile>/` (managed by `secrets` array)
- **Directories**: Stored in `files/<profile>/` (managed by `directories` array)

**File Resolution Order:**
1. Look for file in `files/<profile>/` (profile-specific)
2. Fall back to `files/` (flat structure for backward compatibility)
3. If neither exists, use profile-specific path for new files

**Secrets Array Management:**
- The `secrets` array contains direct paths to source files
- Used for bulk operations with `ordinator secrets add --all`
- Automatically managed by secrets commands
- Supports secure workflow with automatic encryption/decryption

**Benefits:**
- **Profile isolation**: Each profile can have different versions of the same file
- **Clear organization**: Easy to see which files belong to which profile
- **Conflict prevention**: No accidental overwrites between profiles
- **Backward compatibility**: Existing repositories continue to work
- **Secure secrets management**: Dedicated array for encrypted files with bulk operations

## Best Practices

- Keep your `ordinator.toml` up-to-date with your current configuration
- Use the `exclude` field to prevent unnecessary files from being tracked or symlinked
- Regularly review and update encryption patterns and exclusions
- Use the `[secrets]` section to manage secrets securely
- Configure the `[global]` section to set defaults and enable/disable features

## Troubleshooting

### Secrets Scanning Issues

**Scanner detects false positives:**
- The scanner uses regex patterns to detect common secret formats
- Prefixed keys (e.g., `test_api_key`) are intentionally detected
- Use `--verbose` to see detailed information about detected secrets
- Consider excluding files with false positives using `exclude_patterns`

**Scanner skips files:**
- Binary files are automatically skipped to prevent corruption
- Files with permission issues are skipped with warnings
- Large files (>10MB) are skipped to maintain performance
- Check file permissions and ensure files are readable

**Unicode or special character issues:**
- The scanner handles Unicode filenames and content
- Special characters in secret values are supported
- If issues persist, check file encoding (UTF-8 recommended)

**Performance issues:**
- Large repositories may take time to scan
- Use `--profile` to limit scanning to specific profiles
- Consider excluding large binary files with `exclude_patterns`

### Common Error Messages

**"Permission denied" errors:**
- Check file permissions with `ls -la`
- Ensure user has read access to tracked files
- Use `chmod` to fix permissions if needed

**"Binary file detected" warnings:**
- Binary files are automatically skipped for safety
- This is normal behavior to prevent corruption
- Use `exclude_patterns` to skip binary files permanently

**"Large file detected" warnings:**
- Files >10MB are skipped for performance
- This prevents scanning large log files or databases
- Use `exclude_patterns` to skip large files permanently
