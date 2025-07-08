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
- [ ] Implement bootstrap script execution
- [ ] Profile-based script selection
- [ ] Non-interactive mode support
- [ ] Script validation and safety checks

**Tests**:
- [ ] Scripts execute correctly
- [ ] Profile selection works
- [ ] Non-interactive mode functions
- [ ] Safety checks prevent issues

**Acceptance Criteria**:
```bash
ordinator apply --profile work         # Runs bootstrap script
ordinator apply --skip-bootstrap       # Skips bootstrap
```

### 4.2 Package Management Integration
**Priority:** Medium  
**Dependencies:** 4.1  
**Estimated Time:** 2-3 days  
**Testable:** ✅

**Tasks:**
- [ ] Provide a method to pull/export the list of currently installed Homebrew formulae/casks and their versions
- [ ] Add/export this list to the repo/config for reproducibility
- [ ] On `apply`, install all listed formulae/casks at the prescribed versions
- [ ] Ensure reproducible Homebrew environment setup from config

**Tests:**
- [ ] Exported Homebrew package list matches actual installed packages
- [ ] `apply` installs all listed formulae/casks at correct versions
- [ ] Handles missing or outdated packages gracefully
- [ ] Package lists are managed properly in config

**Acceptance Criteria:**
```bash
# User can export Homebrew package list to config
ordinator export-brew
# On apply, all listed formulae/casks are installed at specified versions
ordinator apply --profile work
```

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

**Tests:**
- [ ] Clones repo and initializes config correctly
- [ ] Handles existing directory conflicts safely
- [ ] Works with all supported profiles
- [ ] UX is clear and error messages are helpful

**Acceptance Criteria:**
```bash
ordinator init https://github.com/yourname/dotfiles.git ~/.dotfiles
# Clones the repo to ~/.dotfiles, sets up config, ready for apply
ordinator init https://github.com/yourname/dotfiles.git
# Clones the repo to the current directory by default
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

**Tests:**
- [ ] README is created with correct content on new repo init
- [ ] Existing README is not overwritten
- [ ] Quick-install, AGE key, and documentation links are accurate and copy-pasteable

**Acceptance Criteria:**
```bash
# After ordinator init, repo contains README.md with:
# - Install path
# - Quick-start shell snippet
# - Profile/usage info
# - AGE key warning and path
# - Links to Ordinator project and docs
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

**Tests:**
- [ ] Adding the same file to multiple profiles stores separate copies
- [ ] Applying a profile symlinks the correct version for that profile
- [ ] No accidental overwrites between profiles
- [ ] Backward compatibility for existing flat file structure

**Acceptance Criteria:**
```bash
ordinator add ~/.zshrc --profile work
# stores as files/work/.zshrc

ordinator add ~/.zshrc --profile laptop
# stores as files/laptop/.zshrc

ordinator apply --profile work
# symlinks files/work/.zshrc to ~/.zshrc
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

**Acceptance Criteria:**
```bash
ordinator uninstall --profile work --restore-backups
# Removes all symlinks for 'work' profile and restores backups if available
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

**Tests**:
- [ ] Commands are parsed correctly
- [ ] Scripts are generated properly
- [ ] Safety validation works
- [ ] No sudo commands are executed

**Acceptance Criteria**:
```bash
ordinator generate-script --profile work
# Creates ordinator-system.sh for manual execution
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

**Tests**:
- [ ] macOS commands are handled correctly
- [ ] System preferences are managed
- [ ] Cross-platform compatibility maintained

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

**Tests**:
- [ ] Sync operations work correctly
- [ ] Rebase functionality works
- [ ] Auto-push functions properly

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

**Tests**:
- [ ] JSON output is valid
- [x] Logging levels work correctly
- [ ] Progress indicators function
- [x] Documentation is comprehensive and accurate
- [ ] MAN page is generated, installed, and accessible via `man ordinator`

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

**Tests**:
- [x] Homebrew installation works
- [ ] Cargo installation works
- [x] Package installs correctly
- [x] All features function after installation

### 7.2 Curl Install Script
**Priority**: Medium  
**Dependencies**: 7.1  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [ ] Create `curl | sh` installer
- [ ] Silent mode support
- [ ] Dependency checking

**Tests**:
- [ ] Installer works correctly
- [ ] Silent mode functions
- [ ] Dependencies are checked

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