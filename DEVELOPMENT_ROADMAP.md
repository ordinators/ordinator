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
- [ ] Support `ordinator init --repo <repo-url> [target-dir]`, with `[target-dir]` as a positional argument (defaulting to the current directory if omitted, matching `git clone` behavior). The repository URL is only provided via the `--repo` flag, not as a positional argument.
- [ ] Add `--repo` flag to `ordinator init` for remote cloning (deprecated in favor of positional argument, but may be supported for backward compatibility)
- [ ] Prompt for or accept target directory
- [ ] Clone the specified repository safely (with overwrite checks)
- [ ] Set up configuration and profiles from the cloned repo
- [ ] Optionally support branch/tag selection
- [ ] Integrate with existing bootstrap and apply flows
- [ ] Enhance `ordinator push` to accept a `--repo` or `--remote` URL, set the remote if not already configured, and push to that remote. This reduces reliance on a pre-installed git executable and improves onboarding for new users.
- [ ] Add interactive prompts for repository URL input and directory selection
- [ ] Implement progress indicators for cloning and setup operations
- [ ] Enhance error messages for network issues and authentication problems
- [ ] Add autocompletion for repository URLs and directory paths

**Tests:**
- [ ] Clones repo and initializes config correctly
- [ ] Handles existing directory conflicts safely
- [ ] Works with all supported profiles
- [ ] UX is clear and error messages are helpful
- [ ] Interactive prompts work for repository setup
- [ ] Progress indicators display during cloning operations
- [ ] Autocompletion works for repository URLs and paths

**Acceptance Criteria:**
```bash
ordinator init https://github.com/yourname/dotfiles.git ~/.dotfiles
# Clones the repo to ~/.dotfiles, sets up config, ready for apply
ordinator init https://github.com/yourname/dotfiles.git
# Clones the repo to the current directory by default
# Interactive prompts guide user through setup
# Progress indicators show cloning status
```

### 4.4 Auto-Generated README with Quick-Install & Secrets Instructions
**Priority:** Medium  
**Dependencies:** 1.1, 4.3  
**Estimated Time:** 1 day  
**Testable:** ✅

**Tasks:**
- [ ] Generate a `README.md` file on `ordinator init` if one does not exist
- [ ] Include ideal install path and quick-start shell snippet
- [ ] Add a section about the AGE key, its required location, and security warning
- [ ] Document recommended profiles and bootstrap usage
- [ ] Add links to the Ordinator project and documentation
- [ ] Allow user to customize the README template (optional)
- [ ] Add a shell one-liner for installation to the generated README
- [ ] Include profile table, bootstrap explanation, troubleshooting, and security notes in README
- [ ] Add interactive mode for customizing README template
- [ ] Implement preview functionality to show generated README before saving
- [ ] Add colorized output for highlighting important sections in generated README

**Tests:**
- [ ] README is created with correct content on new repo init
- [ ] Existing README is not overwritten
- [ ] Quick-install, AGE key, and documentation links are accurate and copy-pasteable
- [ ] Interactive mode works for README customization
- [ ] Preview functionality displays README correctly
- [ ] Colorized output renders properly in different terminals

**Acceptance Criteria:**
```bash
# After ordinator init, repo contains README.md with:
# - Install path
# - Quick-start shell snippet
# - Profile/usage info
# - AGE key warning and path
# - Links to Ordinator project and docs
# Interactive prompts allow README customization
# Preview shows generated content before saving
```

### 4.5 Profile-Specific File Storage and Add Command Enhancement
**Priority:** Medium  
**Dependencies:** 2.1, 4.1  
**Estimated Time:** 2 days  
**Testable:** ✅

**Tasks:**
- [ ] Enhance `ordinator add` to support profile-specific file storage
- [ ] When adding a file with `--profile`, store it in `files/<profile>/` subdirectory
- [ ] Update config to track the correct source file for each profile
- [ ] Ensure symlinking logic uses the correct profile-specific file
- [ ] Update documentation and usage examples
- [ ] Add interactive prompts for profile selection when adding files
- [ ] Implement progress indicators for file copying and organization
- [ ] Enhance error handling for file conflicts between profiles
- [ ] Add colorized output for file operations and profile status

**Tests:**
- [ ] Adding the same file to multiple profiles stores separate copies
- [ ] Applying a profile symlinks the correct version for that profile
- [ ] No accidental overwrites between profiles
- [ ] Backward compatibility for existing flat file structure
- [ ] Interactive prompts work for profile selection
- [ ] Progress indicators display during file operations
- [ ] Error handling works for file conflicts

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
```

### 4.6 Uninstall and Restore Original Configuration
**Priority:** Medium  
**Dependencies:** 2.2, 4.1  
**Estimated Time:** 2 days  
**Testable:** ✅

**Tasks:**
- [ ] Implement `ordinator uninstall` command
- [ ] Remove all symlinks created by Ordinator for selected profile(s)
- [ ] Optionally restore original files from backups
- [ ] Support dry-run and force options
- [ ] Prompt for config and repo cleanup (optional)
- [ ] Update documentation and usage examples
- [ ] Add interactive confirmation for destructive operations
- [ ] Implement progress indicators for backup restoration
- [ ] Add colorized output for showing what will be removed/restored
- [ ] Enhance dry-run mode with detailed preview of uninstall actions

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
- [ ] Uninstall removes all symlinks for a profile
- [ ] Backups are restored if requested
- [ ] No data loss if backups are missing
- [ ] Dry-run shows correct actions
- [ ] Interactive confirmations work for destructive operations
- [ ] Progress indicators display during restoration
- [ ] Colorized output shows removal/restoration preview

**Acceptance Criteria:**
```bash
ordinator uninstall --profile work --restore-backups
# Removes all symlinks for 'work' profile and restores backups if available
# Interactive prompts confirm destructive operations
# Progress indicators show restoration status
# Colorized output previews actions
```

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