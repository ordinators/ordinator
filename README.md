# Ordinator — Dotfiles and Environment Manager for macOS

[![CI](https://github.com/ordinators/ordinator/workflows/CI/badge.svg)](https://github.com/ordinators/ordinator/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Ordinator** is a CLI tool written in Rust for managing macOS dotfiles, system settings, and secrets, allowing users to replicate their environment across machines in a secure, repeatable, and non-interactive way.

## Features

- ✅ **Dotfiles Management** - Track and sync dotfiles in Git with symlink management
- ✅ **Bootstrap Process** - Execute setup scripts and install tools non-interactively
- ✅ **Profile Support** - Environment profiles (work, personal, laptop)
- ✅ **Secrets Management** - Secure secrets using Mozilla SOPS + age encryption
- ✅ **Git Integration** - Git-inspired CLI commands without explicit git invocation
- ✅ **macOS-Specific** - System settings support
- ✅ **Dry-Run Mode** - Preview changes before applying them

## Quick Start

### 1. Start a New Ordinator Dotfiles Repo (First-Time Setup)

```bash
# Install Ordinator via Homebrew
brew install ordinators/ordinator/ordinator

# Initialize a new dotfiles repository in your chosen directory
mkdir -p ~/.dotfiles
cd ~/.dotfiles
ordinator init

# Add your first dotfile
ordinator add ~/.zshrc

# Apply your configuration
ordinator apply

# Commit your changes
ordinator commit -m "Initial commit: track dotfiles with Ordinator"

# Push to GitHub (sets remote if needed)
ordinator push https://github.com/username/dotfiles.git
```

### 2. Replicate Your Dotfiles Repo to Another Device (Onboarding a New Machine)

```bash
# Install Ordinator via Homebrew
brew install ordinators/ordinator/ordinator

# Initialize Ordinator with your remote repo (clones to current directory by default)
ordinator init https://github.com/username/dotfiles.git

# Or specify a target directory
ordinator init https://github.com/username/dotfiles.git ~/.dotfiles

# Or use SSH URL
ordinator init git@github.com:username/dotfiles.git

# Change to the cloned directory if needed
cd ~/.dotfiles

# Apply your configuration (choose profile as needed)
ordinator apply --profile work
```

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
> The AGE key (typically at `~/.config/ordinator/age.key`) is required for decrypting secrets, but should always be kept private and out of version control.

## Repository Structure

When you initialize an Ordinator dotfiles repository, it creates the following structure:

```
dotfiles-repo/
├── .git/                   # Git repository
├── .gitignore              # Auto-generated git ignore rules
├── ordinator.toml          # Configuration file
├── README.md               # Auto-generated README (root)
├── readme_state.json       # README state tracking (root)
├── files/                  # Tracked dotfiles
│   ├── .zshrc
│   ├── .gitconfig
│   └── .config/
├── scripts/                # Generated scripts
│   ├── install.sh          # Repository install script
│   └── bootstrap-*.sh      # Profile bootstrap scripts
└── secrets/                # Encrypted secrets (optional)
    ├── secrets.enc.yaml
    └── config.enc.json
```

**Generated Files:**
- **README.md** - Auto-generated with installation instructions, profiles, and troubleshooting
- **scripts/install.sh** - One-liner installation script for the repository
- **readme_state.json** - Tracks configuration changes for smart README updates
- **scripts/bootstrap-*.sh** - Profile-specific setup scripts (generated during apply)

All generated files are committed to the repository when you run `ordinator commit`.

## Documentation

- [Product Requirements Document](PRD.md) - Complete feature specification
- [Development Roadmap](DEVELOPMENT_ROADMAP.md) - Implementation plan
- [Commands Reference](COMMANDS.md) - Complete CLI command documentation
- [Configuration Guide](CONFIGURATION.md) - Configuration file format and usage
- [Test Plan](TEST_PLAN.md) - Testing strategy

## Contributing & Feedback

Contributions, bug reports, and feature requests are welcome!
- [Open an issue or pull request](https://github.com/ordinators/ordinator/issues)

## License

MIT License - see [LICENSE](LICENSE) for details.