# Ordinator Commands

This document provides detailed information about all available Ordinator CLI commands.

## Global Options

All commands support these global options:

- `--dry-run` - Simulate operations without making changes
- `--verbose` - Enable detailed logging and output
- `--quiet` - Suppress status messages (only show errors)

## Core Commands

### `ordinator init`

Initialize a new dotfiles repository.

```bash
ordinator init [REPO_URL] [TARGET_DIR] [OPTIONS]
```

**Arguments:**
- `REPO_URL` - Repository URL to clone from (GitHub HTTPS or SSH)
- `TARGET_DIR` - Target directory for the repository (defaults to current directory)

**Options:**
- `--profile <PROFILE>` - Profile to use for initialization (when not cloning from repo)
- `--force` - Force overwrite existing directory

**Examples:**
```bash
# Basic initialization (new repository)
ordinator init

# Initialize from remote repository
ordinator init https://github.com/username/dotfiles

# Initialize from SSH repository
ordinator init git@github.com:username/dotfiles

# Initialize to specific directory
ordinator init https://github.com/username/dotfiles ~/my-dotfiles

# Initialize with force overwrite
ordinator init https://github.com/username/dotfiles --force

# Initialize new repository with specific profile
ordinator init --profile work
```

**What it does:**

**For new repositories:**
- Creates `ordinator.toml` configuration file
- Creates `.gitignore` file with security-focused ignore rules
- Initializes Git repository
- Creates `files/` and `scripts/` directories
- Sets up default profiles (default, work, personal)

**For remote repositories:**
- Parses GitHub URLs (HTTPS and SSH formats)
- Attempts Git clone first (for public repositories)
- Falls back to source archive download (for private repositories)
- Validates repository structure (checks for `ordinator.toml`)
- Initializes Git repository if not present
- Guides user to next steps after successful initialization

**Supported URL Formats:**
- HTTPS: `https://github.com/username/repo.git`
- SSH: `git@github.com:username/repo.git`
- Both formats are automatically detected and handled

**Next Steps After Remote Initialization:**
1. Review the configuration: `cat ordinator.toml`
2. Apply the dotfiles: `ordinator apply`
3. Set up secrets (if needed): `ordinator secrets setup`

### `ordinator add`

Add a file to the dotfiles repository.

```bash
ordinator add <PATH> [OPTIONS]
```

**Arguments:**
- `PATH` - File or directory to add (required)

**Options:**
- `--profile <PROFILE>` - Profile to associate with this file

**Examples:**
```bash
# Add file to default profile
ordinator add ~/.zshrc

# Add file to specific profile
ordinator add ~/.gitconfig --profile work

# Add directory
ordinator add ~/.config/nvim
```

**What it does:**
- Copies file to `files/` directory
- Updates configuration to track the file
- Associates file with specified profile
- Respects exclusion patterns from config
- **Automatically scans for plaintext secrets** and warns if found (does not block the operation)

### `ordinator apply`

Apply dotfiles to the current system.

```bash
ordinator apply [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to apply (default: "default")
- `--skip-bootstrap` - Skip bootstrap script generation and validation
- `--skip-secrets` - Skip secrets decryption
- `--skip-brew` - Skip Homebrew package installation
- `--force` - Force overwrite existing files

**Examples:**
```bash
# Apply default profile
ordinator apply

# Apply specific profile
ordinator apply --profile work

# Apply with force overwrite
ordinator apply --force

# Apply without bootstrap, secrets, or brew packages
ordinator apply --skip-bootstrap --skip-secrets --skip-brew

# Apply without brew package installation
ordinator apply --profile work --skip-brew
```

**What it does:**
1. **Generates bootstrap script** for the selected profile (unless `--skip-bootstrap`)
2. **Decrypts secrets** for the profile (unless `--skip-secrets`)
3. **Installs Homebrew packages** defined in the profile (unless `--skip-brew`)
4. **Creates symlinks** for tracked files in home directory
5. **Handles conflicts** with existing files (backup if enabled)
6. **Validates bootstrap script** for safety (Blocked, Dangerous, Warning, Safe)
7. **Prints script path and safety level** (if bootstrap script was generated)
8. **Never executes bootstrap script automatically** (see `ordinator bootstrap`)
9. **Uses `--force`** to overwrite non-symlink conflicts

**Order of Operations:**
The apply command follows a specific order to ensure dependencies are satisfied:
- Homebrew packages are installed **before** symlinks are created
- This prevents broken symlinks to Homebrew-installed tools
- Secrets are decrypted **before** symlinks to ensure encrypted files are available
- Bootstrap scripts are generated **first** for user review and manual execution

**Bootstrap Script Safety Levels:**
- **Safe:** No dangerous commands detected
- **Warning:** Contains potentially risky commands (e.g., `rm -rf <path>`, `chmod 777`)
- **Dangerous:** Contains commands like `sudo` (requires manual review)
- **Blocked:** Contains extremely dangerous commands (e.g., `rm -rf /`), execution is blocked

**Bootstrap Workflow:**
1. Run `ordinator apply` to generate and validate the bootstrap script
2. Review the script path and safety level printed by the command
3. Edit the script as needed using `ordinator bootstrap --edit`
4. Run the script manually when ready (e.g., `bash /path/to/bootstrap.sh`)

---

### `ordinator bootstrap`

Show information and instructions for running or editing the generated bootstrap script for a profile.

```bash
ordinator bootstrap [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to bootstrap (default: "default")
- `--edit` - Open the bootstrap script in $EDITOR (or nano) for editing

**Examples:**
```bash
# Show info for default profile
ordinator bootstrap

# Show info for work profile
ordinator bootstrap --profile work

# Edit the bootstrap script for work profile
ordinator bootstrap --profile work --edit
```

**What it does:**
- Locates the generated bootstrap script for the selected profile
- Validates the script for safety before execution
- Prints the script path and safety level
- Prints warnings if the script is Dangerous or Blocked
- **Never executes the script itself**
- Prints the exact shell command the user should run (e.g., `bash /path/to/bootstrap.sh`)
- Advises the user to review and edit the script before running
- If `--edit` is passed, opens the script in `$EDITOR` (or nano) for editing

**Workflow:**
1. Run `ordinator apply` to generate and validate the bootstrap script
2. Run `ordinator bootstrap --edit` to update the script as needed
3. Review the script and its safety level
4. Run the script manually as instructed (e.g., `bash /path/to/bootstrap.sh`)

---

### `ordinator status`

Show repository and symlink status.

```bash
ordinator status [OPTIONS]
```

**Options:**
- `--verbose` - Show detailed status information

**Examples:**
```bash
# Basic status
ordinator status

# Detailed status with symlink information
ordinator status --verbose
```

**What it does:**
- Shows Git repository status (if Git repo exists)
- Lists all tracked files and their symlink status
- Reports valid symlinks, broken symlinks, and missing files
- Provides summary statistics

### `ordinator repair`

Repair broken symlinks.

```bash
ordinator repair [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to repair (defaults to all profiles)
- `--verbose` - Show detailed repair information

**Examples:**
```bash
# Repair all profiles
ordinator repair

# Repair specific profile
ordinator repair --profile work

# Verbose repair
ordinator repair --verbose
```

**What it does:**
- Detects broken symlinks in tracked files
- Recreates symlinks pointing to correct targets
- Reports repair statistics
- Handles missing source files gracefully

## Git Integration Commands

### `ordinator commit`

Commit changes to the repository.

```bash
ordinator commit -m <MESSAGE>
```

**Options:**
- `-m, --message <MESSAGE>` - Commit message (required)
- `--force` - Skip secrets scanning and commit anyway

**Examples:**
```bash
# Commit with message
ordinator commit -m "Add new dotfiles"

# Commit with descriptive message
ordinator commit -m "Update zsh configuration and add vim settings"

# Force commit (skip secrets scanning)
ordinator commit -m "Update config" --force
```

**What it does:**
- Stages all changes in the dotfiles repository
- **Scans all tracked files for plaintext secrets** (unless `--force` is used)
- **Blocks commit with error code 1 if secrets are found** (unless `--force` is used)
- **Warns if no remote 'origin' is set** (affects README generation)
- Creates Git commit with specified message
- Uses Git repository in dotfiles directory

**Remote Warning:**
If no remote 'origin' is configured, the commit will succeed but show a warning:
```
⚠️  Warning: No remote 'origin' set
   This will cause the README to show placeholder URLs instead of your actual repository URL.
   To fix this, run: ordinator push <your-repo-url>
   Example: ordinator push https://github.com/yourname/dotfiles.git
```

### `ordinator push`

Push changes to remote repository.

```bash
ordinator push [REPO_URL] [OPTIONS]
```

**Arguments:**
- `REPO_URL` - Repository URL to push to (sets remote if not configured)

**Options:**
- `--force` - Force push (use with caution)

**Examples:**
```bash
# Push to current remote
ordinator push

# Push to specific repository (sets remote if not configured)
ordinator push https://github.com/username/dotfiles.git

# Force push to specific repository
ordinator push https://github.com/username/dotfiles.git --force

# Push to SSH repository
ordinator push git@github.com:username/dotfiles.git
```

**What it does:**
- Pushes committed changes to the remote repository
- If repository URL is provided, sets it as the remote 'origin' before pushing
- Uses the currently configured remote if no URL is provided
- Supports force push with `--force` flag
- Automatically configures the remote if not already set

### `ordinator pull`

Pull changes from remote repository.

```bash
ordinator pull [OPTIONS]
```

**Options:**
- `--rebase` - Use rebase strategy instead of merge
- `--no-rebase` - Skip rebase and use merge

**Examples:**
```bash
# Pull changes
ordinator pull

# Pull with rebase
ordinator pull --rebase

# Pull without rebase
ordinator pull --no-rebase
```

**What it does:**
- Fetches changes from remote repository
- Updates local repository
- Uses rebase strategy by default
- Supports merge strategy with --no-rebase
- Updates local dotfiles with remote changes

### `ordinator sync`

Synchronize dotfiles with remote repository.

```bash
ordinator sync [OPTIONS]
```

**Options:**
- `--force` - Force push/pull
- `--no-rebase` - Skip rebase during pull

**Examples:**
```bash
# Basic sync
ordinator sync

# Force sync
ordinator sync --force

# Sync without rebase
ordinator sync --no-rebase
```

**What it does:**
- Pulls changes from remote
- Pushes local changes
- Uses rebase strategy by default
- Supports force push/pull
- Updates local dotfiles with remote changes

## Secrets Management Commands

### `ordinator secrets encrypt`

Encrypt a file using SOPS and age.

```bash
ordinator secrets encrypt <PATH>
```

**Arguments:**
- `PATH` - File or directory to encrypt (required)

**Options:**
- `--profile <PROFILE>` - Profile to associate with this file
- `--dry-run` - Simulate encryption without making changes

**Examples:**
```bash
# Encrypt a single file
ordinator secrets encrypt ~/.ssh/config

# Encrypt a directory
ordinator secrets encrypt ~/.aws

# Encrypt with specific profile
ordinator secrets encrypt ~/.ssh/config --profile work
```

**What it does:**
- Uses SOPS and age for encryption
- Preserves file extensions (e.g., `config.yaml` becomes `config.enc.yaml`)
- Adds `.enc` suffix to encrypted files
- Uses encryption patterns from configuration
- Respects exclusion patterns from configuration
- Creates encrypted files in the same directory as original

### `ordinator secrets decrypt`

Decrypt a file using SOPS.

```bash
ordinator secrets decrypt <PATH>
```

**Arguments:**
- `PATH` - File or directory to decrypt (required)

**Options:**
- `--profile <PROFILE>` - Profile to associate with this file
- `--dry-run` - Simulate decryption without making changes

**Examples:**
```bash
# Decrypt a single file
ordinator secrets decrypt ~/.ssh/config.enc

# Decrypt a directory
ordinator secrets decrypt ~/.aws

# Decrypt with specific profile
ordinator secrets decrypt ~/.ssh/config.enc --profile work
```

**What it does:**
- Uses SOPS for decryption
- Restores original file extensions
- Removes `.enc` suffix from decrypted files
- Uses decryption patterns from configuration
- Respects exclusion patterns from configuration
- Creates decrypted files in the same directory as original

### `ordinator secrets setup`

Set up SOPS and age for secrets management.

```bash
ordinator secrets setup [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to set up (default: "default")
- `--force` - Force overwrite existing configuration
- `--dry-run` - Simulate setup without making changes

**Examples:**
```bash
# Set up SOPS and age for default profile
ordinator secrets setup

# Set up for specific profile
ordinator secrets setup --profile work

# Force overwrite existing configuration
ordinator secrets setup --force
```

**What it does:**
- Checks if SOPS and age are installed (installs via Homebrew if missing)
- Generates age encryption key for the profile
- Creates SOPS configuration file (`.sops.yaml`)
- Updates `ordinator.toml` with secrets configuration
- Sets up encryption patterns and exclusions
- Configures age key file location and SOPS config path

### `ordinator secrets list`

List encrypted files in the repository.

```bash
ordinator secrets list [OPTIONS]
```

**Options:**
- `--paths-only` - Show file paths only (no status table)
- `--profile <PROFILE>` - Profile to list files for
- `--verbose` - Show detailed information about encrypted files

**Examples:**
```bash
# List all encrypted files
ordinator secrets list

# List with file paths only
ordinator secrets list --paths-only

# List encrypted files for specific profile
ordinator secrets list --profile work

# List with detailed information
ordinator secrets list --verbose
```

**What it does:**
- Lists all files matching encryption patterns
- Shows encryption status (Encrypted/Plaintext)
- Can filter by profile
- Shows detailed information with --verbose
- Outputs simple paths with --paths-only

### `ordinator secrets validate`

Validate SOPS and age installation.

```bash
ordinator secrets validate
```

**What it does:**
- Checks if SOPS is installed and in PATH
- Checks if age is installed and in PATH
- Shows installation paths if found
- Provides installation instructions if missing

### `ordinator secrets scan`

Scan for plaintext secrets in tracked files.

```bash
ordinator secrets scan [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to scan (defaults to all profiles)
- `--verbose` - Show detailed information about found secrets

**Examples:**
```bash
# Scan all profiles for secrets
ordinator secrets scan

# Scan specific profile
ordinator secrets scan --profile work

# Verbose scan with detailed information
ordinator secrets scan --verbose
```

**What it does:**
- Scans all tracked files for potential plaintext secrets
- Uses advanced regex-based heuristics to detect passwords, API keys, tokens, etc.
- **Detects secrets with prefixes** (e.g., `test_api_key`, `prod_password`)
- **Handles special characters** in filenames and secret values
- **Supports Unicode filenames** and international character sets
- **Lists secret types found without showing actual values**
- **Always exits with error code 1 if secrets are found**
- Provides actionable feedback for encrypting detected secrets
- **Robust error handling** for permission issues, binary files, and large files

### `ordinator secrets check`

Check SOPS and age installation.

```bash
ordinator secrets check
```

**What it does:**
- Checks if SOPS is installed and in PATH
- Checks if age is installed and in PATH
- Shows installation paths if found
- Provides installation instructions if missing

## Homebrew Package Management Commands

### `ordinator brew export`

Export currently installed Homebrew packages to the configuration.

```bash
ordinator brew export [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to export packages to (default: "default")
- `--dry-run` - Simulate export without making changes

**Examples:**
```bash
# Export packages to default profile
ordinator brew export

# Export packages to work profile
ordinator brew export --profile work

# Simulate export without making changes
ordinator brew export --dry-run
```

**What it does:**
- Exports currently installed Homebrew formulae and casks
- Stores package list in the profile's `homebrew_packages` configuration
- Preserves package versions for reproducible environments
- Updates `ordinator.toml` with the exported package list
- Can be used to capture current Homebrew state for sharing

### `ordinator brew install`

Install Homebrew packages defined in the configuration.

```bash
ordinator brew install [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to install packages for (default: "default")
- `--dry-run` - Simulate installation without making changes

**Examples:**
```bash
# Install packages for default profile
ordinator brew install

# Install packages for work profile
ordinator brew install --profile work

# Simulate installation without making changes
ordinator brew install --dry-run
```

**What it does:**
- Installs all Homebrew packages listed in the profile's `homebrew_packages` configuration
- Uses `brew install` for formulae and `brew install --cask` for casks
- Handles missing packages gracefully (continues with available packages)
- Provides progress feedback during installation
- Can be run independently or as part of `ordinator apply`

### `ordinator brew list`

List Homebrew packages defined in the configuration.

```bash
ordinator brew list [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to list packages for (default: "default")
- `--verbose` - Show detailed package information

**Examples:**
```bash
# List packages for default profile
ordinator brew list

# List packages for work profile
ordinator brew list --profile work

# Show detailed package information
ordinator brew list --verbose
```

**What it does:**
- Lists all Homebrew packages defined in the profile's configuration
- Shows formulae and casks separately
- Displays package versions if specified
- Can show detailed information with --verbose flag
- Useful for reviewing what packages will be installed

### `ordinator readme`

Manage README generation for the dotfiles repository.

```bash
ordinator readme [COMMAND] [OPTIONS]
```

**Subcommands:**

#### `ordinator readme default`

Generate default README if none exists.

```bash
ordinator readme default
```

**What it does:**
- Checks if README.md already exists
- If missing, generates a comprehensive README with:
  - **Quick install instructions** with one-liner curl command
  - **Copy-to-clipboard buttons** for easy command copying
  - **Private repository support** with PAT input form
  - **Profile usage information** with available profiles
  - **AGE key setup guide** for secrets management
  - **Troubleshooting section** for common issues
  - **Security notes** and best practices
- **Generates install script** in `scripts/install.sh` (in the dotfiles repository)
- **Generates README.md** in the root of the dotfiles repository
- **Generates state file** `readme_state.json` in the root of the dotfiles repository
- All generated files are committed to the repository when you run `ordinator commit`
- If README.md exists, shows message and exits

#### `ordinator readme interactive`

Interactive README customization.

```bash
ordinator readme interactive
```

**What it does:**
- Steps through customization options:
  - Repository information
  - Installation path
  - Profile configuration
  - AGE key settings
  - README sections to include
- Shows preview before saving
- Allows editing generated content

#### `ordinator readme preview`

Preview generated README before saving.

```bash
ordinator readme preview
```

**What it does:**
- Generates README content (with current config)
- **Uses actual repository URL** if remote is set, otherwise shows placeholder
- Displays formatted preview with all sections
- **Generates install script** in `scripts/install.sh` (in the dotfiles repository)
- Shows interactive copy buttons and PAT input form
- Doesn't save to file automatically
- Provides instructions for saving the README

#### `ordinator readme edit`

Edit existing README in $EDITOR.

```bash
ordinator readme edit
```

**What it does:**
- Opens existing README.md in $EDITOR
- If README.md doesn't exist, generates default first
- Handles missing $EDITOR gracefully
- Preserves user customizations

**Examples:**
```bash
# Generate default README
ordinator readme default

# Interactive customization
ordinator readme interactive

# Preview what would be generated
ordinator readme preview

# Edit existing README
ordinator readme edit
```

**Configuration:**
README generation can be configured in `ordinator.toml`:

```toml
[readme]
auto_update = false  # Enable automatic updates
update_on_changes = ["profiles", "bootstrap"]  # Specific triggers
```

**Auto-Update Behavior:**
- **Manual Mode** (default): Users get notifications when README may need updating
- **Auto Mode**: README automatically regenerates when config changes
- **Smart Detection**: Only updates when profiles, bootstrap, or AGE key info changes
- **File Locations**: All generated files are placed in the dotfiles repository:
  - `README.md` - Root of the repository
  - `scripts/install.sh` - Install script for the repository
  - `readme_state.json` - State tracking file (root of repository)
- **Git Integration**: Generated files are automatically committed when you run `ordinator commit`

## Management Commands

### `ordinator remove`

Remove a file from the dotfiles repository.

```bash
ordinator remove <PATH>
```

**Arguments:**
- `PATH` - File or directory to remove (required)

**Examples:**
```bash
# Remove file from tracking
ordinator remove ~/.zshrc

# Remove directory
ordinator remove ~/.config/nvim
```

**What it does:**
- Removes file from configuration tracking
- Optionally removes file from `files/` directory
- Updates all profiles that reference the file

### `ordinator profiles`

List available profiles.

```bash
ordinator profiles [OPTIONS]
```

**Options:**
- `--verbose` - Show detailed profile information

**Examples:**
```bash
# List profiles
ordinator profiles

# Detailed profile information
ordinator profiles --verbose
```

**What it does:**
- Lists all configured profiles
- Shows profile descriptions and settings
- Reports enabled/disabled status

**Examples:**
```bash
# Generate default system script
ordinator generate-script

# Generate script for specific profile
ordinator generate-script --profile work

# Specify output file
ordinator generate-script -o setup-system.sh
```

**What it does:**
- Generates shell script with system-level commands
- Includes `defaults write` commands and other system settings
- Creates script for manual execution (doesn't run automatically)
- Supports profile-specific system configurations

## Utility Commands

### `ordinator help`

Show help information.

```bash
ordinator help [COMMAND]
```

**Examples:**
```bash
# Show general help
ordinator help

# Show help for specific command
ordinator help apply
ordinator help secrets encrypt
```

### `ordinator version`

Show version information.

```bash
ordinator version
```

**What it does:**
- Displays Ordinator version
- Shows build information and dependencies

## Configuration

Commands use configuration from `ordinator.toml` file. Key configuration options:

- `default_profile` - Default profile for commands
- `create_backups` - Whether to create backups before changes
- `auto_push` - Whether to auto-push after successful operations
- `exclude` - Global exclusion patterns

See [Configuration Guide](CONFIGURATION.md) for detailed configuration options.

## Examples

### Complete Workflow

```bash
# Initialize repository
ordinator init --remote https://github.com/username/dotfiles

# Add configuration files
ordinator add ~/.zshrc
ordinator add ~/.gitconfig --profile work
ordinator add ~/.config/nvim

# Export Homebrew packages for reproducible environments
ordinator brew export --profile work
ordinator brew export --profile personal

# Apply configuration (includes Homebrew package installation)
ordinator apply --profile work

# Commit and push changes
ordinator commit -m "Initial dotfiles setup with Homebrew packages"
ordinator push
```

### Profile Management

```bash
# Create work profile
ordinator init --profile work

# Add work-specific files
ordinator add ~/.ssh/config --profile work
ordinator add ~/.config/company --profile work

# Apply work profile
ordinator apply --profile work
```

### Secrets Management

```bash
# Set up SOPS and age for secrets management
ordinator secrets setup --profile work

# Check SOPS and age installation
ordinator secrets check

# Add files (automatically scans for secrets)
ordinator add ~/.ssh/config --profile work
ordinator add ~/.config/api_keys.json --profile work

# Scan for any remaining secrets
ordinator secrets scan --profile work

# Encrypt sensitive files
ordinator secrets encrypt ~/.ssh/config
ordinator secrets encrypt ~/.config/api_keys.json

# List encrypted files
ordinator secrets list
ordinator secrets list --paths-only

# Commit changes (automatically scans for secrets)
ordinator commit -m "Add encrypted configuration files"

# Decrypt files when needed
ordinator secrets decrypt ~/.ssh/config.enc

# Apply with secrets decryption
ordinator apply
```

### Homebrew Package Management

```bash
# Export current Homebrew packages to configuration
ordinator brew export --profile work

# List packages that will be installed
ordinator brew list --profile work

# Install packages for a profile
ordinator brew install --profile work

# Apply configuration with Homebrew packages
ordinator apply --profile work

# Skip Homebrew installation during apply
ordinator apply --profile work --skip-brew
```

### System Setup

```bash
# Generate system setup script
ordinator generate-script --profile work

# Review and run manually
cat ordinator-system.sh
sudo ./ordinator-system.sh
```

## Troubleshooting

### Common Issues

**"No configuration file found"**
- Run `ordinator init` to create configuration
- Ensure you're in the correct directory

**"Target already exists and is not a symlink"**
- Use `--force` flag to overwrite existing files
- Check what's at the target location first

**"No Git repository found"**
- Run `ordinator init` to initialize Git repository
- Ensure you're in the dotfiles directory

**"Source file not found"**
- Check that managed files exist in `files/` directory
- Verify file paths in configuration

### Secrets Troubleshooting

**"SOPS is not installed or not found in PATH"**
- Run `ordinator secrets setup` to install SOPS and age
- Ensure Homebrew is installed: `brew install sops age`

**"No age key file configured. Run 'ordinator secrets setup' first"**
- Run `ordinator secrets setup --profile <profile>` to generate keys
- Check that the age key file exists and is readable

**"Encryption failed"**
- Verify SOPS and age are properly installed
- Check that the age key file is valid
- Ensure the file you're encrypting exists and is readable

**"Decryption failed"**
- Verify the encrypted file exists and has `.enc` extension
- Check that the age key file matches the one used for encryption
- Ensure SOPS configuration is correct

**"No files match the encryption patterns"**
- Check your `encrypt_patterns` configuration in `ordinator.toml`
- Verify files are in the expected locations
- Use `ordinator secrets list --verbose` for detailed information

**"Plaintext secrets detected in tracked files"**
- Run `ordinator secrets scan` to see which files contain secrets
- Use `ordinator secrets encrypt <file>` to encrypt detected secrets
- Use `--force` flag with commit to override scanning: `ordinator commit -m "message" --force`

### Debug Mode

Use `--verbose` flag for detailed output:

```bash
ordinator apply --verbose
ordinator status --verbose
ordinator repair --verbose
```

### Dry Run Mode

Test commands without making changes:

```bash
ordinator apply --dry-run
ordinator add ~/.zshrc --dry-run
```

## Best Practices

1. **Use Profiles** - Organize files by environment (work, personal, laptop)
2. **Backup Enabled** - Keep `create_backups = true` for safety
3. **Test Changes** - Use `--dry-run` before applying changes
4. **Regular Commits** - Commit changes frequently with descriptive messages
5. **Secure Secrets** - Encrypt sensitive files before adding to repository
6. **System Scripts** - Use `generate-script` for system-level changes

## See Also

- [Product Requirements Document](PRD.md) - Complete feature specification
- [Configuration Guide](CONFIGURATION.md) - Configuration file format and usage
- [Development Roadmap](DEVELOPMENT_ROADMAP.md) - Implementation plan
- [Test Plan](TEST_PLAN.md) - Testing strategy 