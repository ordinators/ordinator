# Ordinator Test Suite

This directory contains the comprehensive test suite for Ordinator, organized into focused test modules that cover different aspects of the CLI functionality.

## Test Architecture

### Common Test Infrastructure (`common.rs`)
- **Test Environment Setup**: Provides `setup_test_environment_with_config()` for isolated test environments
- **Command Creation**: `create_ordinator_command()` helper for consistent CLI testing
- **Environment Variable Management**: `EnvVarGuard` for safe environment variable manipulation
- **Configuration Helpers**: Utilities for creating test configurations and asserting results

### Test Isolation
All tests use temporary directories and isolated environments to prevent interference:
- `ORDINATOR_HOME` set to temporary directory
- `ORDINATOR_CONFIG` points to test-specific configuration
- `ORDINATOR_TEST_MODE` enables test-specific behavior
- Automatic cleanup when tests complete

## Test Modules Overview

### Core CLI Commands

#### `init.rs` - Repository Initialization
- **Purpose**: Tests the `ordinator init` command functionality
- **Coverage**: 
  - New repository creation
  - Remote repository cloning
  - Configuration file generation
  - Profile setup and validation
  - Error handling for invalid inputs

#### `add.rs` - File Addition and Updates
- **Purpose**: Tests the `ordinator add` command and bulk operations
- **Coverage**:
  - Adding individual files to profiles
  - Bulk updates with `--all` flag
  - Error handling for untracked files
  - Profile-specific file management
  - Exclusion pattern validation
  - **Hash-based filename mapping and file_mappings logic**
  - **Test helpers and assertions expect hash-based filenames and mappings**

#### `apply.rs` - Configuration Application
- **Purpose**: Tests the `ordinator apply` command for deploying configurations
- **Coverage**:
  - Symlink creation and management
  - Profile-specific file deployment
  - Bootstrap script execution
  - Secrets decryption during apply
  - Conflict resolution and backup creation
  - **Hash-based filename mapping and file_mappings logic**
  - **Test helpers and assertions expect hash-based filenames and mappings**

#### `watch.rs` - File Tracking
- **Purpose**: Tests the `ordinator watch` command for starting file tracking
- **Coverage**:
  - Adding files to profile tracking
  - Directory tracking
  - Profile selection and validation
  - Conflict detection between profiles

#### `unwatch.rs` - File Untracking
- **Purpose**: Tests the `ordinator unwatch` command for stopping file tracking
- **Coverage**:
  - Removing files from profile tracking
  - Cleanup of tracked files
  - Profile-specific removal

### Secrets Management

#### `secrets.rs` - Secure File Management
- **Purpose**: Tests the complete secrets workflow and encryption system
- **Coverage**:
  - `secrets watch` and `secrets unwatch` commands
  - `secrets add` with secure workflow
  - Bulk operations with `secrets add --all`
  - SOPS and age encryption integration
  - Secrets array management in configuration
  - Plaintext secrets detection and scanning
  - Mock encryption/decryption for testing
  - Error handling for encryption failures
  - **Hash-based filename mapping and file_mappings logic**
  - **Test helpers and assertions expect hash-based filenames and mappings**

### Git Integration

#### `commit.rs` - Git Commit Operations
- **Purpose**: Tests the `ordinator commit` command and Git integration
- **Coverage**:
  - Commit message handling
  - Automatic secrets scanning during commit
  - Force commit bypass for secrets scanning
  - Git repository state management

#### `push.rs` - Git Push Operations
- **Purpose**: Tests the `ordinator push` command
- **Coverage**:
  - Remote repository pushing
  - Remote URL configuration
  - Force push operations

#### `pull.rs` - Git Pull Operations
- **Purpose**: Tests the `ordinator pull` command
- **Coverage**:
  - Remote repository pulling
  - Merge and rebase strategies

#### `sync.rs` - Git Sync Operations
- **Purpose**: Tests the `ordinator sync` command
- **Coverage**:
  - Pull then push operations
  - Conflict resolution

### System Management

#### `bootstrap.rs` - Bootstrap Script Management
- **Purpose**: Tests bootstrap script generation and execution
- **Coverage**:
  - Script generation for profiles
  - Safety validation of generated scripts
  - Manual script editing capabilities
  - Script execution instructions

#### `uninstall.rs` - System Uninstallation
- **Purpose**: Tests the `ordinator uninstall` command
- **Coverage**:
  - Symlink removal
  - Backup restoration
  - Profile-specific uninstallation
  - Interactive confirmations

#### `status.rs` - System Status
- **Purpose**: Tests the `ordinator status` command
- **Coverage**:
  - Repository status reporting
  - Symlink status checking
  - File tracking status
  - Detailed status information

### Package Management

#### `brew.rs` - Homebrew Integration
- **Purpose**: Tests Homebrew package management features
- **Coverage**:
  - Package export from current system
  - Package installation for profiles
  - Package listing and management
  - Profile-specific package tracking

### Profile Management

#### `profiles.rs` - Profile System
- **Purpose**: Tests profile creation and management
- **Coverage**:
  - Profile creation and deletion
  - Profile configuration management
  - Profile switching and validation
  - Profile-specific settings

### Documentation

#### `readme.rs` - README Generation
- **Purpose**: Tests README generation and management
- **Coverage**:
  - Default README generation
  - Interactive README customization
  - README preview functionality
  - README editing capabilities

### Advanced Testing

#### `edge_cases.rs` - Edge Case Testing
- **Purpose**: Tests unusual scenarios and edge cases
- **Coverage**:
  - Unicode and special character handling
  - Large file processing
  - Permission edge cases
  - Network failure scenarios
  - Malformed configuration handling

#### `meta.rs` - Meta Test Infrastructure
- **Purpose**: Tests the test infrastructure itself
- **Coverage**:
  - Test environment validation
  - Mock system integration
  - Test helper function verification
  - Cross-module test coordination

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Module
```bash
cargo test --test add
cargo test --test secrets
```

### Run Edge Cases Tests
```bash
cargo test --test edge_cases
```

### Run with Verbose Output
```bash
cargo test -- --nocapture
```

### Run with Coverage
```bash
cargo tarpaulin --out Html
```

## Test Conventions

### File Naming
- Test files are named after the CLI command they test (e.g., `add.rs` for `ordinator add`)
- Edge case tests are in `edge_cases.rs`
- Infrastructure tests are in `meta.rs`

### Test Function Naming
- Functions follow the pattern `test_<command>_<scenario>()`
- Error cases use `test_<command>_<scenario>_errors()`
- Edge cases use descriptive names like `test_unicode_filename_handling()`

### Test Isolation
- Each test creates its own temporary directory
- Environment variables are managed with `EnvVarGuard`
- Mock binaries are created for external dependencies
- Tests clean up after themselves automatically

### Mock System
- External tools (SOPS, age, Git) are mocked for testing
- Mock binaries are created in temporary directories
- Test-specific behavior is controlled via environment variables

## Test Coverage Goals

- **Unit Tests**: Test individual functions and components
- **Integration Tests**: Test command-line workflows end-to-end
- **Edge Case Tests**: Test unusual scenarios and error conditions
- **Security Tests**: Test secrets management and encryption workflows
- **Performance Tests**: Test with large files and complex configurations

## Contributing to Tests

When adding new features:
1. Add tests to the appropriate module
2. Follow the existing test patterns and conventions
3. Ensure tests are isolated and don't interfere with each other
4. Add edge case tests for error conditions
5. Update this README if adding new test modules 