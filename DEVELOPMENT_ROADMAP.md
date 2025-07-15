# Ordinator Development Roadmap

## Overview
This roadmap breaks down the Ordinator project into actionable, testable chunks. Each chunk should be implemented as a complete, testable feature before moving to the next.

---

## Phase 1: Foundation & Core Infrastructure 🏗️

### 1.1 Basic CLI Framework & Configuration System
**Priority**: Critical  
**Dependencies**: None  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [x] Implement basic CLI command parsing (init, help, version)
- [x] Create configuration file loading/saving (`ordinator.toml`)
- [x] Add profile support in configuration
- [x] Implement dry-run mode flag handling
- [x] Add basic logging setup

**Tests**:
- [x] CLI commands parse correctly
- [x] Configuration files load/save properly
- [x] Profile switching works
- [x] Dry-run mode is respected
- [x] Logging outputs correctly
- [x] CLI integration tests are fully isolated and use .env for config/test mode

**Acceptance Criteria**:
```bash
ordinator --help                    # Shows help
ordinator init --dry-run           # Runs in dry-run mode
ordinator --version                # Shows version
```

### 1.2 Git Integration (Basic)
**Priority**: Critical  
**Dependencies**: 1.1  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [x] Implement `ordinator init` with Git repository creation
- [x] Add remote repository support
- [x] Implement `ordinator status` (Git status)
- [x] Basic Git operations (commit, push, pull)

**Tests**:
- [x] Repository initialization works
- [x] Remote repositories are added correctly
- [x] Git status shows correct information
- [x] Basic Git operations work
- [x] CLI integration tests use per-test config and are reliable

**Acceptance Criteria**:
```bash
ordinator init --remote https://github.com/user/dotfiles.git
ordinator status                    # Shows Git status
ordinator commit -m "message"       # Commits changes
ordinator push                      # Pushes to remote
```

---

## Phase 2: File Management & Symlinking 📁

### 2.1 Dotfiles Tracking System
**Priority**: High  
**Dependencies**: 1.1, 1.2  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [x] Implement `ordinator add <file>` command
- [x] Add file tracking to configuration
- [x] Profile-based file filtering
- [x] File exclusion patterns
- [x] Backup existing files before symlinking

**Tests**:
- [x] Dotfiles can be added to profiles
- [x] Error cases for add (file missing, already tracked, profile missing) are handled
- [x] CLI integration tests are isolated and pass reliably
- [x] Apply/backup logic is tested and passes; all tests use ORDINATOR_HOME for isolation

**Acceptance Criteria**:
```bash
ordinator add ~/.zshrc --profile work
ordinator add ~/.gitconfig
ordinator list-files --profile work    # Shows tracked files
```

### 2.2 Symlink Management
**Priority**: High  
**Dependencies**: 2.1  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [x] Implement symlink creation with backup
- [x] Handle nested directories (`.config`, `Library/Preferences`)
- [x] Symlink validation and repair
- [x] Conflict resolution (existing files/symlinks)

**Tests**:
- [x] Symlinks are created correctly
- [x] Nested directories are handled
- [x] Conflicts are resolved properly
- [x] Existing files are backed up

**Acceptance Criteria**:
```bash
ordinator apply --profile work         # Creates symlinks
ordinator status                       # Shows symlink status
ordinator repair                       # Fixes broken symlinks
```

**Major Improvements Completed**:
- ✅ Fixed Apply command to create new symlinks (was only repairing existing ones)
- ✅ Added proper conflict resolution with `--force` flag
- ✅ Enhanced repair command to detect and fix broken symlinks
- ✅ Improved error messages and debug output
- ✅ Added comprehensive test coverage for symlink scenarios
- ✅ Fixed test isolation issues with `ORDINATOR_HOME` environment variable

---

## Phase 3: Secrets Management 🔐

### 3.1 SOPS Integration
**Priority**: High  
**Dependencies**: 1.1  
**Estimated Time**: 3-4 days  
**Testable**: ✅

**Tasks**:
- [x] Detect SOPS and age installation
- [x] Implement `ordinator secrets encrypt <file>`
- [x] Implement `ordinator secrets decrypt <file>`
- [x] Add encrypted file patterns to config
- [x] Log decryption events

**Tests**:
- [x] SOPS/age detection works
- [x] File encryption/decryption works
- [x] Logging captures events correctly
- [x] Pattern matching works

**Acceptance Criteria**:
```bash
ordinator secrets encrypt secrets.yaml
ordinator secrets decrypt secrets.enc.yaml
ordinator secrets list                 # Shows encrypted files
```

### 3.2 Plaintext Secrets Detection
**Priority**: Medium  
**Dependencies**: 3.1  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [x] Implement secrets detection heuristics
- [x] Add warnings for plaintext secrets
- [x] Integration with file tracking
- [x] Add CLI command with detailed reporting
- [x] Exit with error code when secrets found
- [x] Show secret types found (never actual values)
- [x] Auto-scan files when adding to tracking
- [x] Add --force flag to skip secrets scanning
- [x] Block commits when secrets detected (unless --force)
- [x] Scan all tracked files on commit

**Tests**:
- [x] Secrets are detected correctly
- [x] Warnings are shown appropriately
- [x] False positives are minimized
- [x] CLI command works with verbose output
- [x] Auto-scan works when adding files
- [x] Force flag skips secrets scanning
- [x] Commit blocks when secrets found
- [x] Enhanced test coverage with 182 tests
- [x] Fixed all clippy warnings
- [x] Achieved 59.09% overall coverage

---

## Phase 4: Bootstrap System 🚀

### 4.1 Bootstrap Script Execution
**Priority**: High  
**Dependencies**: 1.1, 2.1  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [x] Implement bootstrap script generation and validation
- [x] Profile-based script selection
- [x] Script validation and safety checks
- [x] CLI command for presenting script info (`ordinator bootstrap`)
- [x] Documentation and usage examples updated

**Tests**:
- [x] Scripts are generated and validated correctly
- [x] Profile selection works
- [x] Safety checks prevent issues
- [x] CLI and integration tests for bootstrap workflow

**Acceptance Criteria**:
```bash
ordinator apply --profile work         # Generates and validates bootstrap script
ordinator apply --skip-bootstrap       # Skips bootstrap script generation
ordinator bootstrap --profile work     # Shows script path, safety level, and command to run
# Ordinator never executes the script itself; user must run it manually
# Only --profile and --edit are supported for ordinator bootstrap
```

**Status:**
- Implementation, documentation, and tests complete as of [date]. Ordinator only generates and validates bootstrap scripts, never executes them.

### 4.2 Package Management Integration
**Priority:** Medium  
**Dependencies:** 4.1  
**Estimated Time:** 2-3 days  
**Testable:** ✅

**Tasks:**
- [x] Provide a method to pull/export the list of currently installed Homebrew formulae/casks and their versions
- [x] Add/export this list to the repo/config for reproducibility
- [x] On `apply`, install all listed formulae/casks at the prescribed versions
- [x] Ensure reproducible Homebrew environment setup from config
- [x] Add progress indicators for package installation operations
- [x] Implement interactive confirmation for package installation decisions
- [x] Add colorized output for package status and installation progress
- [x] Enhance error handling for package installation failures

**Tests:**
- [x] Exported Homebrew package list matches actual installed packages
- [x] `apply` installs all listed formulae/casks at correct versions
- [x] Handles missing or outdated packages gracefully
- [x] Package lists are managed properly in config
- [x] Progress indicators display correctly during package operations
- [x] Interactive prompts work for package installation confirmations
- [x] Colorized output renders properly in different terminal environments

**Acceptance Criteria:**
```bash
# User can export Homebrew package list to config
ordinator brew export --profile work
# On apply, all listed formulae/casks are installed at specified versions
ordinator apply --profile work
# Progress indicators show installation status
# Interactive prompts confirm package installations
```

**Status:**
- ✅ Implementation complete: Added `BrewManager` module with export, install, and list functionality
- ✅ CLI integration complete: Added `ordinator brew` subcommand with export, install, and list commands
- ✅ Configuration integration complete: Added `homebrew_packages` array to profile configuration
- ✅ Apply integration complete: Added `--skip-brew` flag and Homebrew installation during apply
- ✅ Testing complete: Comprehensive unit and integration tests with dummy brew script
- ✅ Documentation complete: Updated COMMANDS.md, CONFIGURATION.md, and DEVELOPMENT_ROADMAP.md
- ✅ Error handling complete: Graceful handling of missing packages and installation failures
- ✅ Progress feedback complete: Installation status and progress indicators implemented

**Completion Statement:** This completes Phase 4.2 (Package Management Integration) and prepares for Phase 4.3 (Remote Repository Bootstrap).

### 4.3 Remote Repository Bootstrap (`ordinator init <repo-url> [target-dir]`)
**Priority:** Medium  
**Dependencies:** 1.1, 2.1, 4.1  
**Estimated Time:** 1-2 days  
**Testable:** ✅

**Tasks:**
- [x] Support `ordinator init <repo-url> [target-dir]` with repository URL as positional argument
- [x] Add `--force` flag for overwriting existing directories
- [x] Clone the specified repository safely (with overwrite checks)
- [x] Set up configuration and profiles from the cloned repo
- [x] Integrate with existing bootstrap and apply flows
- [x] Implement GitHub URL parsing (HTTPS and SSH formats)
- [x] Add fallback to source archive download for private repositories
- [x] Validate repository structure (check for `ordinator.toml`)
- [x] Guide user to next steps after successful initialization
- [x] Add comprehensive error handling for network and authentication issues

**Tests:**
- [x] Clones repo and initializes config correctly
- [x] Handles existing directory conflicts safely
- [x] Works with all supported profiles
- [x] UX is clear and error messages are helpful
- [x] GitHub URL parsing works for both HTTPS and SSH formats
- [x] Source archive download works for private repositories
- [x] Repository validation works correctly

**Acceptance Criteria:**
```bash
ordinator init https://github.com/yourname/dotfiles.git ~/.dotfiles
# Clones the repo to ~/.dotfiles, sets up config, ready for apply
ordinator init git@github.com:yourname/dotfiles.git
# Clones SSH repository to current directory
ordinator init https://github.com/yourname/dotfiles.git --force
# Forces overwrite of existing directory
```

**Completion Statement:** This completes Phase 4.3 (Remote Repository Bootstrap) and prepares for Phase 4.4 (Auto-Generated README).

### 4.4 Auto-Generated README with Quick-Install & Secrets Instructions
**Priority:** Medium  
**Dependencies:** 1.1, 4.3  
**Estimated Time:** 2 days  
**Testable:** ✅

**Tasks:**
- [x] Generate a `README.md` file on `ordinator init` if one does not exist
- [x] Include ideal install path and quick-start shell snippet
- [x] Add a section about the AGE key, its required location, and security warning
- [x] Document recommended profiles and bootstrap usage
- [x] Add links to the Ordinator project and documentation
- [x] Allow user to customize the README template (optional)
- [x] Add a shell one-liner for installation to the generated README
- [x] Include profile table, bootstrap explanation, troubleshooting, and security notes in README
- [x] Add interactive mode for customizing README template
- [x] Implement preview functionality to show generated README before saving
- [x] Add colorized output for highlighting important sections in generated README
- [x] Implement warning system for missing remote 'origin' configuration
- [x] Warn during `ordinator commit` if no remote is set
- [x] Warn during `ordinator push` if no remote is set
- [x] Provide clear instructions to fix the issue
- [x] Use actual repository URL in README generation when remote is set
- [x] Show placeholder URLs when no remote is configured
- [x] Add helpful error messages with specific commands to run

**Tests:**
- [x] README is created with correct content on new repo init
- [x] Existing README is not overwritten
- [x] Quick-install, AGE key, and documentation links are accurate and copy-pasteable
- [x] Interactive mode works for README customization
- [x] Preview functionality displays README correctly
- [x] Colorized output renders properly in different terminals
- [x] Warning is shown when committing without remote
- [x] Warning is shown when pushing without remote
- [x] Commit and push still succeed despite warnings
- [x] No warnings when remote is properly configured
- [x] README uses actual URL when remote is set
- [x] README shows placeholder when no remote is set

**Acceptance Criteria:**
```bash
# After ordinator init, repo contains:
# - README.md (root) with install path, quick-start shell snippet, profile/usage info, AGE key warning and path, links to Ordinator project and docs
# - scripts/install.sh (install script for the repository)
# - readme_state.json (state tracking file in root)
# Interactive prompts allow README customization
# Preview shows generated content before saving

# Without remote set:
ordinator commit -m "Update config"
# Shows warning but succeeds

ordinator push
# Shows warning but succeeds

# With remote set:
ordinator commit -m "Update config"
# No warning, succeeds normally

# README generation uses actual URL when remote is set
ordinator readme default
# Uses actual repository URL instead of placeholder
```

**Completion Statement:** This completes Phase 4.4 (Auto-Generated README with Remote URL Warning System) and prepares for Phase 4.5 (Profile-Specific File Storage).

### 4.5 Profile-Specific File Storage and Add Command Enhancement
**Priority:** Medium  
**Dependencies:** 2.1, 4.1  
**Estimated Time:** 2 days  
**Testable:** ✅

**Tasks:**
- [x] Enhance `ordinator add` to support profile-specific file storage
- [x] When adding a file with `--profile`, store it in `files/<profile>/` subdirectory
- [x] Update config to track the correct source file for each profile
- [x] Ensure symlinking logic uses the correct profile-specific file
- [x] Update documentation and usage examples
- [x] Add interactive prompts for profile selection when adding files
- [x] Implement progress indicators for file copying and organization
- [x] Enhance error handling for file conflicts between profiles
- [x] Add colorized output for file operations and profile status
- [x] Update README generator to dynamically list profiles from ordinator.toml (remove hardcoded 'work', 'personal', 'laptop')
- [x] Ensure README always matches actual profiles in config

**Tests:**
- [x] Adding the same file to multiple profiles stores separate copies
- [x] Applying a profile symlinks the correct version for that profile
- [x] No accidental overwrites between profiles
- [x] Backward compatibility for existing flat file structure
- [x] Interactive prompts work for profile selection
- [x] Progress indicators display during file operations
- [x] Error handling works for file conflicts
- [x] Generated README lists only the profiles present in ordinator.toml
- [x] Changing profiles in config updates README accordingly
- [x] No hardcoded profiles appear in README

**Acceptance Criteria:**
```bash
ordinator add ~/.zshrc --profile work
# stores as files/work/.zshrc

ordinator add ~/.zshrc --profile laptop
# stores as files/laptop/.zshrc

ordinator apply --profile work
# symlinks files/work/.zshrc to ~/.zshrc
# Interactive prompts guide profile selection
# Progress indicators show file operations
# README profile section is always accurate to ordinator.toml
# No mention of profiles that do not exist in the config
```

**Completion Statement:** This completes Phase 4.5 (Profile-Specific File Storage and Add Command Enhancement) and prepares for Phase 4.6 (Uninstall and Restore Original Configuration).

### 4.6 Uninstall and Restore Original Configuration
**Priority:** Medium  
**Dependencies:** 2.2, 4.1  
**Estimated Time:** 2 days  
**Testable:** ✅

**Tasks:**
- [x] Implement `ordinator uninstall` command
- [x] Remove all symlinks created by Ordinator for selected profile(s)
- [x] Optionally restore original files from backups
- [x] Support dry-run and force options
- [x] Prompt for config and repo cleanup (optional)
- [x] Update documentation and usage examples
- [x] Add interactive confirmation for destructive operations
- [x] Implement progress indicators for backup restoration
- [x] Add colorized output for showing what will be removed/restored
- [x] Enhance dry-run mode with detailed preview of uninstall actions

**Backups Details:**
- Backups are created during `ordinator apply` if a file already exists at the target location and backups are enabled (`create_backups = true`).
- The original file is moved to a backup location (e.g., `~/.zshrc.ordinator.bak`) before the symlink is created.
- Backups are typically stored in the same directory as the original file, with a `.ordinator.bak` or similar suffix.
- During uninstall/restore, Ordinator will remove the symlink and, if a backup exists, move it back to the original location.
- If multiple backups exist, the most recent is restored (or the user is prompted).
- If backups are disabled, uninstall will only remove symlinks and not restore originals.
- If no backup exists, the symlink is removed and the user is warned that the original file cannot be restored.
- Best practice: Always enable backups to ensure safe restoration of original files.

**Tests:**
- [x] Uninstall removes all symlinks for a profile
- [x] Backups are restored if requested
- [x] No data loss if backups are missing
- [x] Dry-run shows correct actions
- [x] Interactive confirmations work for destructive operations
- [x] Progress indicators display during restoration
- [x] Colorized output shows removal/restoration preview

**Acceptance Criteria:**
```bash
ordinator uninstall --profile work --restore-backups
# Removes all symlinks for 'work' profile and restores backups if available
# Interactive prompts confirm destructive operations
# Progress indicators show restoration status
# Colorized output previews actions
```

**Completion Statement:** This completes Phase 4.6 (Uninstall and Restore Original Configuration) and prepares for Phase 4.7 (Secrets Workflow Review and Enhancement).

### 4.7 Secrets Workflow Review and Enhancement
**Priority:** High  
**Dependencies:** 3.1, 3.2  
**Estimated Time:** 2-3 days  
**Testable:** ✅

**Tasks:**
- [x] Review current secrets workflow for security issues
- [x] Identify problems with current `add` + `encrypt` workflow
- [x] Design new `ordinator secrets add` command for secure file handling
- [x] Implement `ordinator secrets add <file> --profile <profile>` command
- [x] Ensure encrypted files are never stored in plaintext in repository
- [x] Update documentation to reflect secure workflow
- [x] Add tests for new secure secrets workflow
- [x] Update Quick Start guide with correct workflow
- [x] Add validation to prevent plaintext secrets in repository
- [x] Update commit scanning to detect plaintext secrets more effectively
- [x] Add interactive prompts for secrets workflow decisions
- [x] Enhance error messages for secrets-related operations
- [x] Add `secrets` array to profile configuration for direct source paths
- [x] Implement `ordinator secrets add --all` for bulk re-encryption
- [x] Add `ordinator add --all` for bulk file updates
- [x] Update CLI to make path optional when using `--all` flag
- [x] Update documentation for both `add` and `secrets add` commands

**Current Issues:**
- `ordinator add` stores plaintext files in repository
- `ordinator secrets encrypt` creates encrypted version but leaves plaintext
- Manual cleanup required to remove plaintext files
- Security risk of plaintext secrets in Git repository
- Workflow is complex and error-prone

**New Secure Workflow:**
```bash
# Secure way to add sensitive files
ordinator secrets add ~/.ssh/config --profile work
# 1. Reads source file
# 2. Encrypts in memory
# 3. Saves only encrypted version to repository
# 4. Tracks encrypted file in config
# 5. Never stores plaintext

# Bulk operations for efficiency
ordinator secrets add --all --profile work
# Loops through all files in secrets array and re-encrypts them
```

**Tests:**
- [x] `ordinator secrets add` never stores plaintext in repository
- [x] Encrypted files are properly tracked in configuration
- [x] Commit scanning detects plaintext secrets effectively
- [x] Interactive prompts guide users through secure workflow
- [x] Error messages are clear about security implications
- [x] Backward compatibility maintained for existing workflows
- [x] Integration tests verify secure file handling
- [x] `secrets add --all` loops through secrets array correctly
- [x] `add --all` loops through files array correctly
- [x] Path argument is optional when using `--all` flag
- [x] Bulk operations work without manual file path specification

**Acceptance Criteria:**
```bash
# New secure workflow
ordinator secrets add ~/.ssh/config --profile work
# Result: Only encrypted file stored in repository

# Bulk operations
ordinator add --all --profile work
ordinator secrets add --all --profile work
# Result: All tracked files updated without manual path specification

# Validation
ordinator commit -m "Add sensitive files"
# Blocks commit if plaintext secrets detected
# Provides clear guidance on how to fix
```

**Major Improvements Completed:**
- ✅ **Secrets Array Management**: Added dedicated `secrets` array in profile configuration for direct source paths
- ✅ **Bulk Operations**: Implemented `--all` flag for both `add` and `secrets add` commands
- ✅ **CLI Enhancement**: Made path argument optional when using `--all` flag
- ✅ **Secure Workflow**: `secrets add` reads source files directly and encrypts in memory
- ✅ **Documentation Updates**: Updated COMMANDS.md and CONFIGURATION.md to reflect new features
- ✅ **Test Coverage**: Added comprehensive tests for bulk operations and secrets array management
- ✅ **Backward Compatibility**: Maintained existing workflows while adding new features

**Completion Statement:** This completes Phase 4.7 (Secrets Workflow Review and Enhancement) and prepares for Phase 4.8 (AGE Key Prompting During Apply).

### 4.8 AGE Key Prompting During Apply & Key Rotation Tracking
**Priority:** High  
**Dependencies:** 3.1, 4.4  
**Estimated Time:** 1-2 days  
**Testable:** ✅

**Tasks:**
- [x] Update configuration structs and TOML format to support `created_on` in each profile
- [x] Update configuration structs and TOML format to support `key_rotation_interval_days` in `[secrets]`
- [x] Track age key creation date (`created_on`) in each profile in `ordinator.toml`
- [x] Add `key_rotation_interval_days` option to `[secrets]` in `ordinator.toml`
- [x] On age key creation/rotation, set or update `created_on` for the profile
- [x] On secrets/age-related CLI commands, check if the key is older than the configured interval
- [x] Prompt the user to rotate the key if the interval has elapsed
- [x] Backward compatibility: if `created_on` is missing, fall back to file creation time or prompt to set
- [x] Add tests for metadata tracking, interval checking, and prompting logic
- [x] Document the feature in PRD.md and README
- [x] Detect missing AGE key during `ordinator apply --profile <name>`
- [x] Implement interactive prompting for AGE key setup
- [x] Support two scenarios:
  - [x] Scenario 1: Generate new AGE key (first-time setup)
  - [x] Scenario 2: Import existing AGE key (multi-machine replication)
- [x] Add key validation for imported keys
- [x] Implement secure key storage with proper permissions (600)
- [x] Generate corresponding SOPS config for imported keys
- [x] Update `ordinator.toml` with key and config paths
- [x] Add clear user guidance and error messages
- [x] Ensure backward compatibility with existing workflows
- [x] Add comprehensive test coverage for both scenarios
- [x] Update documentation to reflect new behavior
- [x] Update README quick start section to remove `ordinator secrets setup` requirement since apply will handle AGE key setup through prompts

**Scenario 1 - New Key Generation:**
```
❌ AGE key not found for profile 'work'
Would you like to generate a new AGE key? (y/N): y
✅ AGE key generated successfully
   Key stored at: ~/.config/ordinator/age/work.txt
   SOPS config created at: ~/.config/ordinator/.sops.work.yaml
```

**Scenario 2 - Import Existing Key:**
```
❌ AGE key not found for profile 'work'
Do you have an existing AGE key to import? (y/N): y
Please paste your AGE private key (it will be stored securely):
AGE-SECRET-KEY-1abc123...
✅ AGE key imported successfully
   Key stored at: ~/.config/ordinator/age/work.txt
   SOPS config created at: ~/.config/ordinator/.sops.work.yaml
```

**Tests:**
- [x] Configuration structs properly serialize/deserialize `created_on` field in profiles
- [x] Configuration structs properly serialize/deserialize `key_rotation_interval_days` field in secrets
- [x] Age key creation sets `created_on` timestamp in the correct profile
- [x] Age key rotation updates `created_on` timestamp in the correct profile
- [x] Interval checking works correctly for different rotation periods (30, 90, 180, 365 days)
- [x] Warning prompts appear when key age exceeds configured interval
- [x] No warning appears when key age is within configured interval
- [x] Backward compatibility: missing `created_on` falls back to file creation time
- [x] Backward compatibility: missing `key_rotation_interval_days` uses default behavior (no warnings)
- [x] CLI commands check key age on secrets/age operations
- [x] Integration tests cover complete key rotation workflow
- [x] Missing AGE key is detected during apply
- [x] Interactive prompts work for both scenarios
- [x] New key generation works correctly
- [x] Existing key import validates key format
- [x] Imported keys are stored securely with proper permissions
- [x] SOPS config is generated correctly for imported keys
- [x] `ordinator.toml` is updated with correct paths
- [x] Apply continues successfully after key setup
- [x] Error handling works for invalid keys
- [x] Backward compatibility maintained for existing workflows
- [x] Integration tests cover both scenarios
- [x] CLI tests verify prompting behavior

**Acceptance Criteria:**
```bash
# Scenario 1: First-time setup (IMPLEMENTED)
ordinator apply --profile work
# Detects missing age key and prompts for setup
# Guides user through new key generation or key import
# Continues with apply process after key setup

# Scenario 2: Multi-machine replication (IMPLEMENTED)
ordinator apply --profile work
# Detects missing age key and prompts for setup
# Allows importing existing key from another machine
# Validates key format and stores securely

# Error handling (IMPLEMENTED)
ordinator apply --profile work
# Handles invalid key format gracefully
# Provides clear error messages and recovery guidance
# Allows skipping secrets with --skip-secrets flag

# Key mismatch handling (IMPLEMENTED)
ordinator apply --profile work
# Detects when encrypted secrets can't be decrypted with current key
# Provides options: skip file, cancel operation, or import correct key
# Continues with other apply operations even if some secrets can't be decrypted
```

**Current Status:**
- ✅ Key rotation tracking and warnings are implemented
- ✅ Age key creation with timestamps is implemented
- ✅ Interactive age key setup during apply is implemented
- ✅ Missing key detection during apply is implemented
- ✅ Key import functionality is implemented

**Additional Tasks for Key Mismatch Handling:**
- [x] Detect when new age key cannot decrypt existing encrypted secrets
- [x] Implement graceful error handling for key mismatch scenarios
- [x] Provide clear user guidance when decryption fails due to key mismatch
- [x] Add option to skip secrets decryption and continue with other apply operations
- [x] Show list of files that cannot be decrypted with the new key
- [x] Add warning messages explaining the key mismatch issue
- [x] Implement `--skip-secrets` flag handling in apply command
- [x] Add tests for key mismatch detection and graceful degradation
- [x] Update documentation to explain key mismatch scenarios and recovery options

**Key Mismatch Scenario:**
```
⚠️  Warning: Found encrypted secrets that were created with a different key
   The following files cannot be decrypted with the new key:
   - secrets/config.yaml
   - secrets/credentials.json
   
   To decrypt these secrets, you need the original key.
   The apply will continue without decrypting these secrets.
   
   To fix this:
   1. Get the original age key
   2. Import it using: ordinator age setup --profile work
   3. Re-apply: ordinator apply --profile work
   
   Or continue without secrets: ordinator apply --profile work --skip-secrets
```

**Completion Statement:** This completes Phase 4.8 (AGE Key Prompting During Apply) with all core functionality implemented. The interactive age key setup, key mismatch handling, and `--skip-secrets` flag are all working. Documentation has been updated to explain the new flows. This prepares for Phase 4.9 (Enhanced README with Homebrew Packages Section).

### 4.9 Enhanced README with Homebrew Packages Section
**Priority:** Medium  
**Dependencies:** 4.4  
**Estimated Time:** 1 day  
**Testable:** ✅

**Tasks:**
- [x] Remove all JavaScript-based PAT input and copy buttons from the generated README. Replace with plain text instructions for using a Personal Access Token (PAT) with private repositories, ensuring compatibility with GitHub's Markdown rendering.
- [x] Implement true interactive README customization:
    - [x] Prompt the user for project name, description, and key sections to include.
    - [x] Allow users to select which profiles to include.
    - [x] Preview the README before saving.
    - [x] Optionally allow editing in $EDITOR before final save.
- [x] Replace the placeholder in `interactive_customization` with real interactive logic.
- [x] Add Homebrew packages section to README generator
- [x] Create profile-specific collapsible HTML sections with package links to formulae.brew.sh
- [x] Read `homebrew_formulas` and `homebrew_casks` from each profile in config
- [x] Generate separate sections for each profile that has packages
- [x] Sort packages alphabetically within each profile
- [x] Link each package to `https://formulae.brew.sh/formula/{package_name}`
- [x] Use profile-appropriate emojis (💼 work, 🏠 personal, 💻 laptop, ⚙️ default)
- [x] Use collapsible sections for both profiles and Homebrew packages sections
- [x] Keep profiles and Homebrew packages as separate, focused sections
- [x] Ensure backward compatibility with existing READMEs
- [x] Update `ordinator brew export` to use `brew leaves -r` instead of `brew list` for formulas
- [x] Add `brew list --cask` support to export casks separately
- [x] Store formulas and casks separately in TOML configuration
- [x] Update `ordinator brew install` to install both formulas and casks on apply
- [x] Add separate `homebrew_formulas` and `homebrew_casks` fields to profile configuration
- [x] Update TOML structure to distinguish between formulas and casks
- [x] Modify export process to call both commands and merge results
- [x] Update install process to handle both formulas and casks installation
- [x] Add tests for separate formula and cask handling
- [x] Update documentation to reflect new formula/cask separation
- [x] Remove backward compatibility with existing `homebrew_packages` field
- [x] Update existing configurations to use new formula/cask structure

**HTML Structure:**

**Homebrew Packages Section:**
```html
## Homebrew Packages

<details>
  <summary><strong>💼 Work Profile Packages</strong></summary>
  <div style="margin-top:10px; padding:10px; border:1px solid #ddd; border-radius:8px;">
    <p>
      <a href="https://formulae.brew.sh/formula/git" target="_blank">git</a> •
      <a href="https://formulae.brew.sh/formula/neovim" target="_blank">neovim</a> •
      <a href="https://formulae.brew.sh/formula/ripgrep" target="_blank">ripgrep</a> •
      <a href="https://formulae.brew.sh/formula/sops" target="_blank">sops</a> •
      <a href="https://formulae.brew.sh/formula/age" target="_blank">age</a>
    </p>
  </div>
</details>

<details>
  <summary><strong>🏠 Personal Profile Packages</strong></summary>
  <div style="margin-top:10px; padding:10px; border:1px solid #ddd; border-radius:8px;">
    <p>
      <a href="https://formulae.brew.sh/formula/git" target="_blank">git</a> •
      <a href="https://formulae.brew.sh/formula/alacritty" target="_blank">alacritty</a> •
      <a href="https://formulae.brew.sh/formula/karabiner-elements" target="_blank">karabiner-elements</a>
    </p>
  </div>
</details>
```

**Enhanced Profiles Section:**
```html
## Profiles

This repository contains the following profiles:

<details>
  <summary><strong>💼 Work Profile</strong> - Work environment configuration</summary>
  <div style="margin-top:10px; padding:10px; border:1px solid #ddd; border-radius:8px;">
    <p><strong>Files:</strong> <code>.zshrc</code>, <code>.gitconfig</code>, <code>.ssh/config</code></p>
    <p><strong>Directories:</strong> <code>.config/nvim/</code>, <code>.config/company/</code></p>
    <p><strong>Bootstrap Script:</strong> <code>scripts/bootstrap-work.sh</code></p>
  </div>
</details>

<details>
  <summary><strong>🏠 Personal Profile</strong> - Personal environment configuration</summary>
  <div style="margin-top:10px; padding:10px; border:1px solid #ddd; border-radius:8px;">
    <p><strong>Files:</strong> <code>.zshrc</code>, <code>.gitconfig</code></p>
    <p><strong>Directories:</strong> <code>.config/alacritty/</code>, <code>.config/karabiner/</code></p>
  </div>
</details>
```

**Tests:**
- [x] README generation includes Homebrew packages when present in config
- [x] Each profile with packages gets its own collapsible section
- [x] Packages are sorted alphabetically within each profile
- [x] Each package links to correct formulae.brew.sh URL
- [x] Profile-appropriate emojis are used (💼 work, 🏠 personal, 💻 laptop, ⚙️ default)
- [x] Collapsible sections render correctly in GitHub
- [x] README generation works without Homebrew packages (no section)
- [x] Enhanced profiles section shows files and directories
- [x] Both profiles and Homebrew packages sections use collapsible sections
- [x] Profiles and Homebrew packages are separate, focused sections
- [x] State tracking updates when Homebrew packages change
- [x] Backward compatibility maintained for existing READMEs
- [x] `brew export` correctly calls `brew leaves -r` for formulas
- [x] `brew export` correctly calls `brew list --cask` for casks
- [x] Formulas and casks are stored separately in TOML configuration
- [x] `brew install` installs both formulas and casks during apply
- [x] TOML structure properly distinguishes between formulas and casks
- [x] Export process handles both commands and merges results correctly
- [x] Install process handles both formulas and casks installation
- [x] Breaking change: `homebrew_packages` field replaced with separate formula/cask fields
- [x] Integration tests cover separate formula and cask workflows
- [x] Migration guide provided for existing configurations

**Acceptance Criteria:**
```bash
# With comprehensive profile information
ordinator readme default
# Generates README with:
# - Enhanced profiles section (files, directories, bootstrap scripts only)
# - Separate Homebrew packages section with formulae links
# - Both sections use collapsible sections for clean organization

# Without Homebrew packages in config
ordinator readme default
# Generates README without Homebrew packages section
# Enhanced profiles section still shows files and directories

# Enhanced Homebrew export with separate formulas and casks
ordinator brew export --profile work
# Calls 'brew leaves -r' for formulas (user-installed only)
# Calls 'brew list --cask' for casks
# Stores formulas and casks separately in TOML
# Breaking change: Replaces existing homebrew_packages field

# Enhanced Homebrew install during apply
ordinator apply --profile work
# Installs both formulas and casks from profile
# Uses separate installation commands for each type

# When Homebrew packages are added/removed from profiles
ordinator brew export --profile work
ordinator readme default
# README automatically updates to reflect new packages in work profile
```

**Completion Statement:** This completes Phase 4.9 (Enhanced README with Homebrew Packages Section) and prepares for Phase 4.10 (Hash-Based Filename Mapping).

### 4.10 Hash-Based Filename Mapping
**Priority:** Medium  
**Dependencies:** 2.1, 3.1, 4.7  
**Estimated Time:** 2-3 days  
**Testable:** ✅

**Tasks:**
- [ ] Implement hash-based filename generation for file collision prevention
- [ ] Add hash generation function using SHA-256 truncated to 6 characters
- [ ] Implement file mapping system in TOML configuration
- [ ] Update `ordinator watch` to use hash-based filenames
- [ ] Update `ordinator secrets watch` to use hash-based filenames
- [ ] Modify file storage to use `files/<profile>/<hash>_<filename>` pattern
- [ ] Add `file_mappings` field to `ProfileConfig` structure
- [ ] Update `ordinator add` to work with hash-based filenames
- [ ] Update `ordinator secrets add` to work with hash-based filenames
- [ ] Modify apply command to use mappings for correct symlink creation
- [ ] Implement backward compatibility for existing files without hashes
- [ ] Add migration logic for existing repositories
- [ ] Update bulk operations (`--all` flags) to work with mappings
- [ ] Add comprehensive test coverage for hash-based file management
- [ ] Update documentation to reflect new file naming system

**Hash Generation:**
```rust
fn generate_file_hash(path: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[0..6].to_string()
}
```

**Example Workflow:**
```bash
# Regular files
ordinator watch ~/.config/app/config.txt --profile work
# Hash: "a1b2c3" -> stores as: files/work/a1b2c3_config.txt
# Mapping: "a1b2c3_config.txt" -> "~/.config/app/config.txt"

ordinator watch ~/Documents/config.txt --profile work
# Hash: "d4e5f6" -> stores as: files/work/d4e5f6_config.txt
# Mapping: "d4e5f6_config.txt" -> "~/Documents/config.txt"

# Secrets
ordinator secrets watch ~/.ssh/config --profile work
# Hash: "9f8e7d" -> stores as: files/work/9f8e7d_config.enc
# Mapping: "9f8e7d_config.enc" -> "~/.ssh/config"
```

**TOML Structure:**
```toml
[profiles.work]
files = ["~/.config/app/config.txt", "~/Documents/config.txt"]
secrets = ["~/.ssh/config", "~/.aws/credentials"]
file_mappings = {
  "a1b2c3_config.txt" = "~/.config/app/config.txt",
  "d4e5f6_config.txt" = "~/Documents/config.txt",
  "9f8e7d_config.enc" = "~/.ssh/config",
  "1a2b3c_credentials.enc" = "~/.aws/credentials"
}
```

**Tests:**
- [ ] Hash generation produces consistent results for same path
- [ ] Different paths produce different hashes
- [ ] File collisions are prevented with hash-based naming
- [ ] `ordinator watch` creates correct hash-based filenames
- [ ] `ordinator secrets watch` creates correct hash-based filenames
- [ ] `ordinator add` updates files with hash-based names
- [ ] `ordinator secrets add` updates encrypted files with hash-based names
- [ ] Apply command creates correct symlinks using mappings
- [ ] Bulk operations work with hash-based file system
- [ ] Backward compatibility maintained for existing files
- [ ] Migration handles existing repositories gracefully
- [ ] Error handling works for hash collisions (extremely unlikely)
- [ ] Integration tests cover complete workflow

**Acceptance Criteria:**
```bash
# No more filename collisions
ordinator watch ~/.config/app/config.txt --profile work
ordinator watch ~/Documents/config.txt --profile work
# Result: files/work/a1b2c3_config.txt and files/work/d4e5f6_config.txt

# Secrets work with hash-based naming
ordinator secrets watch ~/.ssh/config --profile work
ordinator secrets add --all --profile work
# Result: files/work/9f8e7d_config.enc

# Apply uses mappings correctly
ordinator apply --profile work
# Result: Correct symlinks created using TOML mappings
```

**Benefits:**
- ✅ **No filename collisions**: Hash ensures unique filenames
- ✅ **Deterministic**: Same path always generates same hash
- ✅ **Consistent across machines**: Hash is path-based, not random
- ✅ **Backward compatible**: Existing files continue to work
- ✅ **Works for both files and secrets**: Unified approach
- ✅ **Collision resistant**: SHA-256 truncated to 6 chars = 16.7M combinations

**Completion Statement:** This completes Phase 4.10 (Hash-Based Filename Mapping) and prepares for Phase 5 (System Commands & Script Generation).

---

## Phase 5: System Commands & Script Generation ⚙️

### 5.1 System Command Parsing
**Priority**: Medium  
**Dependencies**: 1.1  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [ ] Parse system commands from config
- [ ] Validate command safety
- [ ] Generate system script (`ordinator-system.sh`)
- [ ] Never execute sudo commands directly
- [ ] Add interactive mode for step-by-step system command execution
- [ ] Implement progress indicators for script generation and validation
- [ ] Add colorized output for safety level indicators (Safe/Warning/Dangerous/Blocked)
- [ ] Enhance error messages for command validation failures

**Tests**:
- [ ] Commands are parsed correctly
- [ ] Scripts are generated properly
- [ ] Safety validation works
- [ ] No sudo commands are executed
- [ ] Interactive mode works for command execution
- [ ] Progress indicators display during script generation
- [ ] Colorized output renders safety levels correctly

**Acceptance Criteria**:
```bash
ordinator generate-script --profile work
# Creates ordinator-system.sh for manual execution
# Interactive mode guides through command execution
# Progress indicators show generation status
# Colorized output highlights safety levels
```

### 5.2 macOS-Specific Features
**Priority**: Medium  
**Dependencies**: 5.1  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [ ] `defaults write` command support
- [ ] macOS-specific utilities
- [ ] System preference management
- [ ] Add interactive prompts for system preference changes
- [ ] Implement preview mode to show what system changes will be made
- [ ] Add confirmation dialogs for potentially destructive system changes
- [ ] Add colorized output for system preference status

**Tests**:
- [ ] macOS commands are handled correctly
- [ ] System preferences are managed
- [ ] Cross-platform compatibility maintained
- [ ] Interactive prompts work for system changes
- [ ] Preview mode displays changes correctly
- [ ] Confirmation dialogs prevent accidental changes

**Acceptance Criteria**:
```bash
ordinator system --preview
# Shows what system changes will be made
ordinator system --interactive
# Guides through system preference changes
```

---

## Phase 6: Advanced Features & Polish ✨

### 6.1 Advanced Git Operations
**Priority**: Low  
**Dependencies**: 1.2  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [ ] `ordinator sync` (pull + push)
- [ ] `ordinator pull --rebase`
- [ ] Auto-push after successful apply
- [ ] Git conflict resolution
- [ ] Add progress indicators for sync, pull, push operations
- [ ] Implement interactive conflict resolution for merge conflicts
- [ ] Add colorized output for Git status and diff information

**Tests**:
- [ ] Sync operations work correctly
- [ ] Rebase functionality works
- [ ] Auto-push functions properly
- [ ] Progress indicators display during Git operations
- [ ] Interactive conflict resolution works
- [ ] Colorized output renders Git information correctly

### 6.2 Enhanced Logging & Output
**Priority**: Low  
**Dependencies**: 1.1  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [ ] JSON output format
- [x] Verbose logging levels
- [ ] Progress indicators
- [x] Error reporting improvements
- [x] Command documentation
- [ ] Generate and install MAN page for CLI usage
- [ ] Add structured logging with better log organization
- [ ] Implement log file output for persistent logging capability
- [ ] Add performance metrics and timing information for operations
- [ ] Add silent mode for CI/CD integration

**Tests**:
- [ ] JSON output is valid
- [x] Logging levels work correctly
- [ ] Progress indicators function
- [x] Documentation is comprehensive and accurate
- [ ] MAN page is generated, installed, and accessible via `man ordinator`
- [ ] Structured logging works correctly
- [ ] Log file output functions properly
- [ ] Performance metrics are accurate
- [ ] Silent mode works for automated environments

### 6.3 Advanced CLI Features
**Priority**: Low  
**Dependencies**: 1.1, 6.2  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [ ] Add comprehensive shell autocompletion (bash, zsh, fish)
- [ ] Implement full interactive mode for enhanced CLI experience
- [ ] Add configuration validation with interactive repair capabilities
- [ ] Implement update notifications to check for updates and notify users
- [ ] Add plugin system for extensible command system (future consideration)
- [ ] Implement shell integration for better integration with user's shell environment

**Tests**:
- [ ] Shell autocompletion works for bash, zsh, and fish
- [ ] Interactive mode provides enhanced user experience
- [ ] Configuration validation and repair functions correctly
- [ ] Update notifications work without being intrusive
- [ ] Shell integration improves user workflow
- [ ] Plugin system is extensible (future)

**Acceptance Criteria**:
```bash
# Autocompletion works in user's shell
ordinator <TAB>  # Shows available commands
ordinator add <TAB>  # Shows available files

# Interactive mode provides guided experience
ordinator --interactive

# Configuration validation and repair
ordinator validate-config --repair

# Update notifications
ordinator --check-updates
```

---

## Phase 7: Installation & Distribution 📦

### 7.1 Installation Methods (Homebrew & Cargo)
**Priority**: Medium  
**Dependencies**: All previous phases  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [x] Create Homebrew formula
- [x] Package for distribution
- [x] Installation script (Homebrew formula and documented Homebrew install command)
- [ ] Document and support installation via Cargo (`cargo install ordinator`)
- [ ] Add installation verification with post-install checks and setup
- [ ] Implement proper uninstall cleanup when removing Ordinator
- [ ] Add self-update capability for easy updates
- [ ] Add progress indicators for installation process

**Tests**:
- [x] Homebrew installation works
- [ ] Cargo installation works
- [x] Package installs correctly
- [x] All features function after installation
- [ ] Installation verification works correctly
- [ ] Uninstall cleanup removes all components
- [ ] Self-update functionality works
- [ ] Progress indicators display during installation

### 7.2 Curl Install Script
**Priority**: Medium  
**Dependencies**: 7.1  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [ ] Create `curl | sh` installer
- [ ] Silent mode support
- [ ] Dependency checking
- [ ] Add progress indicators for download and installation
- [ ] Enhance dependency checking with better error messages
- [ ] Add installation verification post-install

**Tests**:
- [ ] Installer works correctly
- [ ] Silent mode functions
- [ ] Dependencies are checked
- [ ] Progress indicators display during download and installation
- [ ] Enhanced error messages help users resolve issues
- [ ] Installation verification validates successful installation

---

## Testing Strategy 🧪

### Unit Tests
- Each module should have comprehensive unit tests
- Mock external dependencies (Git, SOPS, file system)
- Test error conditions and edge cases

### Integration Tests
- Test complete workflows end-to-end
- Use temporary directories and repositories
- Test cross-platform compatibility

### Manual Testing Checklist
- [ ] Fresh installation on macOS
- [ ] Profile switching
- [ ] Secrets encryption/decryption
- [ ] Bootstrap script execution
- [ ] System script generation
- [ ] Error handling and recovery

---

## Development Guidelines 📋

### For Each Chunk:
1. **Start with tests** - Write tests first (TDD approach)
2. **Implement incrementally** - Small, working pieces
3. **Test thoroughly** - Unit + integration tests
4. **Document changes** - Update README and examples
5. **Commit frequently** - Small, focused commits

### Quality Gates:
- [ ] All tests pass
- [ ] Code coverage > 80%
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Examples work correctly

### Definition of Done:
- Feature is implemented
- Tests are written and passing
- Documentation is updated
- Examples are provided
- Code is reviewed (if applicable)
- Feature is tested manually

---

## Estimated Timeline 📅

- **Phase 1**: 3-5 days
- **Phase 2**: 4-6 days  
- **Phase 3**: 4-6 days
- **Phase 4**: 4-6 days
- **Phase 5**: 4-6 days
- **Phase 6**: 3-5 days
- **Phase 7**: 2-4 days

**Total Estimated Time**: 24-38 days (4-6 weeks)

This timeline assumes focused development time and can be adjusted based on availability and priorities. 