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
- `--force` - Force push (use with caution)

**Examples:**
```bash
# Normal push
ordinator push

# Force push
ordinator push --force
```

**What it does:**
- Pushes committed changes to remote repository
- Uses `origin` as default remote
- Supports force push for overwriting remote changes

### `ordinator pull`

Pull changes from remote repository.

```bash
ordinator pull [OPTIONS]
```

**Options:**
- `--rebase` - Rebase on pull

**Examples:**
```bash
# Normal pull
ordinator pull

# Pull with rebase
ordinator pull --rebase
```

**What it does:**
- Fetches and merges changes from remote repository
- Supports rebase strategy for clean history
- Updates local dotfiles with remote changes

### `ordinator sync`

Sync with remote repository (pull then push).

```bash
ordinator sync [OPTIONS]
```

**Options:**
- `--force` - Force push after sync

**Examples:**
```bash
# Normal sync
ordinator sync

# Sync with force push
ordinator sync --force
```

**What it does:**
- Pulls latest changes from remote
- Pushes local changes to remote
- Ensures local and remote are in sync

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

## Secrets Management

### `ordinator secrets encrypt`

Encrypt a file with SOPS.

```bash
ordinator secrets encrypt <FILE>
```

**Arguments:**
- `FILE` - File to encrypt (required)

**Examples:**
```bash
# Encrypt secrets file
ordinator secrets encrypt ~/.ssh/id_rsa

# Encrypt configuration with secrets
ordinator secrets encrypt ~/.config/api_keys.json
```

**What it does:**
- Encrypts file using Mozilla SOPS with age encryption
- Preserves file structure and metadata
- Uses configured age key for encryption

### `ordinator secrets decrypt`

Decrypt a file with SOPS.

```bash
ordinator secrets decrypt <FILE>
```

**Arguments:**
- `FILE` - File to decrypt (required)

**Examples:**
```bash
# Decrypt secrets file
ordinator secrets decrypt ~/.ssh/id_rsa.enc

# Decrypt configuration
ordinator secrets decrypt ~/.config/api_keys.json.enc
```

**What it does:**
- Decrypts file using Mozilla SOPS
- Requires valid age key for decryption
- Logs decryption events for audit trail

### `ordinator secrets list`

List encrypted files.

```bash
ordinator secrets list [OPTIONS]
```

**Options:**
- `--paths-only` - Show file paths only

**Examples:**
```bash
# List all encrypted files
ordinator secrets list

# Show only paths
ordinator secrets list --paths-only
```

**What it does:**
- Lists all encrypted files in the repository
- Shows encryption status and metadata
- Respects secrets configuration patterns

## System Integration

### `ordinator generate-script`

Generate system script for manual execution.

```bash
ordinator generate-script [OPTIONS]
```

**Options:**
- `-o, --output <FILE>` - Output file path (default: "ordinator-system.sh")
- `--profile <PROFILE>` - Profile to use (default: "default")

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