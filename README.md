# Ordinator

> Dotfiles and Environment Manager for macOS

Ordinator is a CLI tool written in Rust for managing macOS dotfiles, system settings, and secrets, allowing users to replicate their environment across machines in a secure, repeatable, and non-interactive way — with no GUI wrapper.

## Features

### ✅ Dotfiles Management
- Track user-defined dotfiles in a Git repository
- Symlink dotfiles into the home directory with backup and overwrite protection
- Support nested folders (`.config`, `Library/Preferences`, etc.)

### ✅ Bootstrap Process
- Execute user-defined shell/bootstrap script or TOML-defined commands
- Install tools like Homebrew packages and VS Code extensions
- Support silent/non-interactive mode for automation
- Parse system-level commands but generate separate scripts for manual execution

### ✅ Profile Support
- Define environment profiles (e.g., `work`, `personal`, `laptop`)
- Profile-based filtering of files, overrides, bootstrap steps

### ✅ Secrets Management
- Use Mozilla SOPS with `age` for encrypted secrets files
- Secrets handled **per file**
- Secrets decrypted on-demand during bootstrap
- Log each decryption event (with file path + timestamp)
- Warn users if plaintext secrets are detected in tracked files

### ✅ Git Integration (Streamlined CLI)
- Local repo is initialized or linked to a remote (e.g., GitHub)
- Simple, Git-inspired commands without explicitly invoking `git`:
  - `ordinator init --remote <url>`
  - `ordinator commit -m "msg"`
  - `ordinator push`, `ordinator pull`, `ordinator sync`, `ordinator status`
- Optionally auto-push after successful apply

### ✅ macOS-Specific Enhancements
- Apply `defaults write` tweaks and other system settings
- System-level commands never run automatically — only output to script
- Homebrew Bundle and macOS-specific utilities supported

### ✅ Dry-Run Mode
- Simulate dotfile linking, secrets decryption, bootstrap steps, and script generation
- Nothing is written or run
- `--dry-run` available for all applicable commands
- CLI and optionally JSON output
- Useful for testing, debugging, and trust-building

## Installation

### Via Homebrew (Recommended)
```bash
brew install yourusername/ordinator/ordinator
```

### Via curl script
```bash
curl -fsSL https://raw.githubusercontent.com/yourusername/ordinator/main/install.sh | sh
```

## Quick Start

1. **Initialize a new dotfiles repository:**
   ```bash
   ordinator init --remote https://github.com/yourusername/dotfiles.git
   ```

2. **Add your first dotfile:**
   ```bash
   ordinator add ~/.zshrc
   ```

3. **Commit and push:**
   ```bash
   ordinator commit -m "Add zsh configuration"
   ordinator push
   ```

4. **Apply on a new machine:**
   ```bash
   ordinator apply --profile work
   ```

## Configuration

Create an `ordinator.toml` file in your dotfiles repository:

```toml
[profiles.work]
description = "Work environment configuration"
bootstrap_script = "scripts/bootstrap-work.sh"

[profiles.personal]
description = "Personal environment configuration"
bootstrap_script = "scripts/bootstrap-personal.sh"

[secrets]
age_key_file = "~/.config/ordinator/age.key"
```

## Security

- **Secrets Management**: Uses Mozilla SOPS with age encryption for secure secret handling
- **System Commands**: Never executes sudo commands automatically - generates scripts for manual review
- **Dry-Run Mode**: Always test changes before applying them
- **Logging**: All secret operations are logged with timestamps

## Development

### Prerequisites
- Rust 1.70+
- Git
- SOPS (for secrets management)
- age (for encryption)

### Building
```bash
cargo build --release
```

### Testing
```bash
cargo test
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## Roadmap

- [ ] Migration tools from existing dotfile managers
- [ ] Plugin/hook system
- [ ] Integration with system login scripts
- [ ] Linux support
- [ ] UI wrapper (if demand is high) 