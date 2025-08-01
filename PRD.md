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
- **Hash-based filename mapping:** All tracked files and secrets are stored as `files/<profile>/<hash>_<filename>`, with a mapping in the config to prevent collisions and ensure deterministic, profile-specific storage.

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

### ✅ Homebrew Integration
- Only missing formulas and casks are installed during apply, using a single command for each. Already-installed packages are skipped for efficiency and idempotency.

---

## 5. Design Considerations

- Secrets handling must never expose plaintext unexpectedly
- Sudo-required commands are opt-in and manually executed
- Config must support profiles and non-interactive modes
- **Hash-based mapping ensures collision resistance and deterministic file storage, even for files with the same name in different locations.**
- Clear logs and validation help build user confidence
- CLI must feel intuitive — especially for developers who know Git
- Internals written in Rust for safety, performance, and reliability
- Homebrew installation is optimized for performance and idempotency: only missing packages are installed, reducing redundant operations and improving user experience.

---

## 6. Technical Architecture

- Written in Rust
- CLI parsing via `clap` or similar crate
- Secrets handled with Mozilla `sops` and `age`
- Git management via `git2` crate or shell-out to Git
- Config specified via `ordinator.toml`
- **Hash-based filename mapping for all tracked files and secrets, with mapping table in config**
- Dry-run, logging, and system-script generation core to apply engine
- Installable via Homebrew or `curl | sh` installer
- CI/CD includes automated testing, code formatting, and static analysis
- Homebrew installation logic queries installed packages and forms a single install command for missing formulas and casks.

## Replication Script & Branch Detection (Phase 5.2)

- Ordinator generates a `replicate.sh` script at the root of your dotfiles repository on `ordinator init`.
- The script uses the detected default branch (auto-detected from the remote, or falls back to 'main') for all git operations and URLs.
- The onboarding output and generated README now include a one-liner for quick replication:
  bash <(curl -fsSL https://raw.githubusercontent.com/<username>/<repo>/<branch>/replicate.sh)
- The README and CLI output include a note: "If your repository uses a different default branch (e.g., master), update the one-liner to match your branch name."
- Branch detection is performed using the git2 library and remote HEAD, with fallback to the current branch or 'main'.
- This ensures users are never left with a broken one-liner due to branch mismatch, and onboarding is robust for any repo configuration.

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