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
ordinator init [OPTIONS]
```

**Options:**
- `--remote <URL>` - Remote Git repository URL (e.g., GitHub)
- `--profile <PROFILE>` - Profile to use for initialization (default: "default")

**Examples:**
```bash
# Basic initialization
ordinator init

# Initialize with remote repository
ordinator init --remote https://github.com/username/dotfiles

# Initialize with specific profile
ordinator init --profile work --remote https://github.com/username/work-dotfiles
```

**What it does:**
- Creates `ordinator.toml` configuration file
- Initializes Git repository
- Creates `files/` and `scripts/` directories
- Sets up default profiles (default, work, personal)
- Adds remote repository if specified

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

### `ordinator apply`

Apply dotfiles to the current system.

```bash
ordinator apply [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to apply (default: "default")
- `--skip-bootstrap` - Skip bootstrap script execution
- `--skip-secrets` - Skip secrets decryption
- `--force` - Force overwrite existing files

**Examples:**
```bash
# Apply default profile
ordinator apply

# Apply specific profile
ordinator apply --profile work

# Apply with force overwrite
ordinator apply --force

# Apply without bootstrap or secrets
ordinator apply --skip-bootstrap --skip-secrets
```

**What it does:**
- Creates symlinks for tracked files in home directory
- Handles conflicts with existing files (backup if enabled)
- Executes bootstrap scripts (unless skipped)
- Decrypts secrets (unless skipped)
- Uses `--force` to overwrite non-symlink conflicts

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

**Examples:**
```bash
# Commit with message
ordinator commit -m "Add new dotfiles"

# Commit with descriptive message
ordinator commit -m "Update zsh configuration and add vim settings"
```

**What it does:**
- Stages all changes in the dotfiles repository
- Creates Git commit with specified message
- Uses Git repository in dotfiles directory

### `ordinator push`

Push changes to remote repository.

```bash
ordinator push [OPTIONS]
```

**Options:**
- `--force` - Force push changes
- `--no-rebase` - Skip rebase before push

**Examples:**
```bash
# Push changes
ordinator push

# Force push
ordinator push --force
```

**What it does:**
- Pushes local changes to remote repository
- Uses git push command
- Supports force push option
- Supports rebase strategy for clean history
- Updates local dotfiles with remote changes

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

### `ordinator secrets list`

List encrypted files in the repository.

```bash
ordinator secrets list [OPTIONS]
```

**Options:**
- `--profile <PROFILE>` - Profile to list files for
- `--verbose` - Show detailed information about encrypted files

**Examples:**
```bash
# List all encrypted files
ordinator secrets list

# List encrypted files for specific profile
ordinator secrets list --profile work

# List with detailed information
ordinator secrets list --verbose
```

**What it does:**
- Lists all encrypted files in the repository
- Shows file paths and encryption status
- Can filter by profile
- Shows detailed information with --verbose

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

### `ordinator secrets check`

Check for plaintext secrets in files.

```bash
ordinator secrets check <PATH>
```

**Arguments:**
- `PATH` - File or directory to check (required)

**Options:**
- `--profile <PROFILE>` - Profile to check files for
- `--verbose` - Show detailed information about detected secrets

**Examples:**
```bash
# Check a single file
ordinator secrets check ~/.ssh/config

# Check a directory
ordinator secrets check ~/.aws

# Check with specific profile
ordinator secrets check ~/.ssh/config --profile work
```

**What it does:**
- Scans files for potential plaintext secrets
- Uses configuration patterns to identify sensitive data
- Shows detailed information about detected secrets
- Helps identify files that need encryption

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

# Apply configuration
ordinator apply

# Commit and push changes
ordinator commit -m "Initial dotfiles setup"
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
# Encrypt sensitive files
ordinator secrets encrypt ~/.ssh/id_rsa
ordinator secrets encrypt ~/.config/api_keys.json

# Apply with secrets decryption
ordinator apply
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