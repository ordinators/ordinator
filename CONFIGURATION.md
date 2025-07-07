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
   - Location: `~/.config/sops/{profile}.yaml`
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
✅ SOPS config already exists: ~/.config/sops/work.yaml
✅ SOPS and age setup complete for profile: work
   Age key: ~/.config/age/work.key
   SOPS config: ~/.config/sops/work.yaml
```

### Generated Configuration

After setup, your `ordinator.toml` will include:

```toml
[secrets]
age_key_file = "~/.config/age/work.key"
sops_config = "~/.config/sops/work.yaml"
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
     "**/*.enc.yaml",
     "secrets/excluded/**/*"
   ]
   encryption_format = "{stem}.sops.{ext}"
   encryption_method = "age"
   ```

## Security Best Practices

### Key Management
- **Separate keys per environment**: Use different age keys for work, personal, and laptop profiles
- **Secure key storage**: Store age keys in `~/.config/age/` with restricted permissions (600)
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

# 2. Add sensitive files to tracking
ordinator add ~/.ssh/config --profile work
ordinator add ~/.config/api_keys.json --profile work

# 3. Encrypt sensitive files
ordinator secrets encrypt ~/.ssh/config
ordinator secrets encrypt ~/.config/api_keys.json

# 4. Verify encryption worked
ordinator secrets list

# 5. Apply configuration with decryption
ordinator apply --profile work

# 6. Verify decryption worked
ls -la ~/.ssh/config
```

---

## Best Practices

- Keep your `ordinator.toml` up-to-date with your current configuration
- Use the `exclude` field to prevent unnecessary files from being tracked or symlinked
- Regularly review and update encryption patterns and exclusions
- Use the `[secrets]` section to manage secrets securely
- Configure the `[global]` section to set defaults and enable/disable features
