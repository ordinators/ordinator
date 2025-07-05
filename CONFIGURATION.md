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
enabled = true
description = "Default profile for basic dotfiles"
exclude = ["*.bak"]

[profiles.work]
files = ["~/.ssh/config"]
directories = []
enabled = true
description = "Work environment profile"

[profiles.personal]
files = []
directories = []
enabled = true
description = "Personal environment profile"

[secrets]
age_key_file = "~/.config/age/key.txt"
sops_config = "~/.sops.yaml"
encrypt_patterns = ["secrets/*.yaml"]
exclude_patterns = ["*.bak"]
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
- `directories` (array of strings): List of directories tracked by this profile.
- `bootstrap_script` (string, optional): Path to a bootstrap script for this profile.
- `enabled` (bool): Whether this profile is active.
- `description` (string, optional): Description of the profile.
- `exclude` (array of strings): Glob patterns for files or directories to exclude for this profile (overrides or adds to global exclusions).

### `[secrets]`
- `age_key_file` (string, optional): Path to the age key file for decryption.
  - Must contain a valid age key in the format `age1...`
  - Used for both encryption and decryption operations
  - If not specified, SOPS will use default key locations

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
    - `*.enc.yaml` - Exclude already encrypted YAML files
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

---

## Customizing Your Configuration

- Add or remove profiles by editing the `[profiles.<name>]` sections.
- Add files or directories to profiles to track them with Ordinator.
- Use the `[secrets]` section to specify which files should be encrypted and how secrets are managed.
- Use the `[global]` section to set defaults and enable/disable features.
- Use the `exclude` field in `[global]` or `[profiles.<name>]` to prevent certain files or directories from being tracked or symlinked. Profile-level `exclude` patterns take precedence over global ones.
- Configure encryption patterns and exclusions to control which files are automatically encrypted.
- Customize encryption format and method to match your security requirements.

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
   age_key_file = "~/.config/age/work.key"
   sops_config = "~/.config/sops/work-config.yaml"
   encrypt_patterns = [
     "secrets/**/*.yaml",
     "secrets/**/*.json",
     "*.key"
   ]
   exclude_patterns = [
     "*.bak",
     "*.enc.yaml",
     "secrets/excluded/**/*"
   ]
   encryption_format = "{stem}.sops.{ext}"
   encryption_method = "age"
   ```

---

## Best Practices

- Keep your `