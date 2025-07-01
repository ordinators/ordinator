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
- [ ] Handle nested directories (`.config`, `Library/Preferences`)
- [ ] Symlink validation and repair
- [ ] Conflict resolution (existing files/symlinks)

**Tests**:
- [ ] Symlinks are created correctly
- [ ] Nested directories are handled
- [ ] Conflicts are resolved properly
- [ ] Existing files are backed up

**Acceptance Criteria**:
```bash
ordinator apply --profile work         # Creates symlinks
ordinator status                       # Shows symlink status
ordinator repair                       # Fixes broken symlinks
```

---

## Phase 3: Secrets Management 🔐

### 3.1 SOPS Integration
**Priority**: High  
**Dependencies**: 1.1  
**Estimated Time**: 3-4 days  
**Testable**: ✅

**Tasks**:
- [ ] Detect SOPS and age installation
- [ ] Implement `ordinator secrets encrypt <file>`
- [ ] Implement `ordinator secrets decrypt <file>`
- [ ] Add encrypted file patterns to config
- [ ] Log decryption events

**Tests**:
- [ ] SOPS/age detection works
- [ ] File encryption/decryption works
- [ ] Logging captures events correctly
- [ ] Pattern matching works

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
- [ ] Implement secrets detection heuristics
- [ ] Add warnings for plaintext secrets
- [ ] Integration with file tracking

**Tests**:
- [ ] Secrets are detected correctly
- [ ] Warnings are shown appropriately
- [ ] False positives are minimized

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
**Priority**: Medium  
**Dependencies**: 4.1  
**Estimated Time**: 2-3 days  
**Testable**: ✅

**Tasks**:
- [ ] Homebrew package installation
- [ ] VS Code extension installation
- [ ] Package list management in config

**Tests**:
- [ ] Homebrew packages install correctly
- [ ] VS Code extensions install correctly
- [ ] Package lists are managed properly

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
- [ ] Verbose logging levels
- [ ] Progress indicators
- [ ] Error reporting improvements

**Tests**:
- [ ] JSON output is valid
- [ ] Logging levels work correctly
- [ ] Progress indicators function

---

## Phase 7: Installation & Distribution 📦

### 7.1 Homebrew Integration
**Priority**: Medium  
**Dependencies**: All previous phases  
**Estimated Time**: 1-2 days  
**Testable**: ✅

**Tasks**:
- [ ] Create Homebrew formula
- [ ] Package for distribution
- [ ] Installation script

**Tests**:
- [ ] Homebrew installation works
- [ ] Package installs correctly
- [ ] All features function after installation

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