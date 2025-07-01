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
- `sops_config` (string, optional): Path to the SOPS configuration file.
- `encrypt_patterns` (array of strings): Glob patterns for files to encrypt.
- `exclude_patterns` (array of strings): Glob patterns for files to exclude from encryption.

---

## Customizing Your Configuration

- Add or remove profiles by editing the `[profiles.<name>]` sections.
- Add files or directories to profiles to track them with Ordinator.
- Use the `[secrets]` section to specify which files should be encrypted and how secrets are managed.
- Use the `[global]` section to set defaults and enable/disable features.
- Use the `exclude` field in `[global]` or `[profiles.<name>]` to prevent certain files or directories from being tracked or symlinked. Profile-level `exclude` patterns take precedence over global ones.

---

## Best Practices

- Keep your `