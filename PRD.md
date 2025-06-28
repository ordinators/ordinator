# Ordinator — Dotfiles and Environment Manager for macOS

---

## 1. Project Overview

**Ordinator** is a CLI tool written in Rust for managing macOS dotfiles, system settings, and secrets, allowing users to replicate their environment across machines in a secure, repeatable, and non-interactive way — with no GUI wrapper.

---

## 2. Goals and Objectives

- Manage and version dotfiles in a Git repository with easy syncing
- Securely manage secrets on a **per file** basis using Mozilla SOPS + age encryption
- Support both interactive and non-interactive environment bootstrapping
- Separate user-space config and system-level settings via scripts
- Provide a dry-run mode for safe previews before changes are applied
- Deliver easy installation via Homebrew and a `curl | sh` script

---

## 3. Target Audience

- Developers setting up or syncing macOS environments
- Security-conscious users managing secrets and config securely
- Power users who want automation without fragile scripting

---

## 4. Key Features

### ✅ Dotfiles Management

- Track user-defined dotfiles in a Git repository (e.g., `~/.dotfiles`)
- Symlink dotfiles into the home directory with backup and overwrite protection
- Support nested folders (`.config`, `Library/Preferences`, etc.)

### ✅ Bootstrap Process

- Execute user-defined shell/bootstrap script or TOML-defined commands
- Install tools like Homebrew packages and VS Code extensions
- Support silent/non-interactive mode for automation
- Parse system-level commands (e.g., `sudo defaults write`) but:
  - **Do not execute them**
  - Instead, generate a separate `ordinator-system.sh` script for users to run manually

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

---

## 5. Design Considerations

- Secrets handling must never expose plaintext unexpectedly
- Sudo-required commands are opt-in and manually executed
- Config must support profiles and non-interactive modes
- Clear logs and validation help build user confidence
- CLI must feel intuitive — especially for developers who know Git
- Internals written in Rust for safety, performance, and reliability

---

## 6. Technical Architecture

- Written in Rust
- CLI parsing via `clap` or similar crate
- Secrets handled with Mozilla `sops` and `age`
- Git management via `git2` crate or shell-out to Git
- Config specified via `ordinator.toml`
- Dry-run, logging, and system-script generation core to apply engine
- Installable via Homebrew or `curl | sh` installer
- CI/CD includes automated testing, code formatting, and static analysis

---

## 7. Open Questions (Resolved)

| Question | Answer |
|---------|--------|
| **How should secrets be handled?** | ✅ Required, per-file, using `sops` + `age`. |
| **Will a migration tool be included?** | 🔸 Not initially. It's a nice-to-have. |
| **Should bootstrap support non-interactive mode?** | ✅ Yes. Silent setup must be supported. |
| **How should installation work?** | ✅ Via Homebrew and `curl | sh` script (installs Homebrew if needed). |
| **Should install script support silent mode?** | ✅ Yes. `--silent` or `--yes` will bypass prompts. |
| **Should Ordinator log secrets activity?** | ✅ Yes. Log decrypted secrets with path and timestamp. |
| **Should Ordinator detect plaintext secrets?** | ✅ Yes. Warn if plaintext secrets are in tracked files. |
| **Should Ordinator run sudo-required commands directly?** | ❌ No. It generates a `ordinator-system.sh` script instead. |
| **Should non-standard shells be supported (e.g., fish, tcsh)?** | 🔸 Nice-to-have. Not required for MVP. |

---

## 8. Appendices / Future Work

### 📌 Future Enhancements

- Migration/import tools from existing dotfile managers
- Plugin/hook system (pre-apply, post-apply events)
- Integration with system login scripts or launchd agents
- Support for Linux (optional expansion)
- UI wrapper (maybe?) if user demand is high

--- 