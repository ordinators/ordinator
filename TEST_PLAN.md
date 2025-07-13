# Ordinator Test Plan

## 1. Introduction

This document outlines the testing strategy for **Ordinator**, a Rust-based CLI tool for managing macOS dotfiles, secrets, and environment setup. It aims to ensure correctness, security, and user trust through a combination of automated and manual testing.

---

## 2. Testing Objectives

- Verify functional correctness of core features (dotfiles, bootstrap, secrets)
- Ensure secure handling of secrets and prevent accidental plaintext exposure
- Confirm proper handling and generation of system-level scripts (no unauthorized sudo)
- Validate all CLI commands, including dry-run, apply, and sync
- Ensure compatibility with silent/non-interactive setups
- Catch edge cases and regressions early using fuzzing and property-based testing
- Provide clear diagnostics and logs for user debugging

---

## 3. Scope

**In Scope:**

- All Rust modules: config, CLI, dotfiles, bootstrap, secrets, dry-run, logging
- CLI command interface and expected output
- Integration with Git and SOPS
- Shell script generation and validation

**Out of Scope:**

- GUI components (none planned)
- Homebrew formula behavior (beyond confirming integration)

---

## 4. Test Types and Approaches

### 4.1 Unit Testing

- Target: Internal functions and modules (parsers, transformers, logic)
- Tools: Rust's built-in `#[test]` framework
- Run: On every commit/PR via CI

### 4.2 Integration Testing

- Target: Multiple components working together
- Simulate full workflows using temporary directories and mocks
- Tools: `tempfile`, `assert_fs`, Git test repos

### 4.3 CLI Testing

- Target: Full CLI flows (`apply`, `sync`, `status`, etc.)
- Test expected output, flags, errors, and side effects
- Tools: `assert_cmd`, `predicates`

### 4.4 Property-Based Testing

- Target: Config parsers, secret input edge cases
- Tools: `proptest`, `quickcheck`
- Goal: Ensure resilience to unexpected or malformed inputs

### 4.5 Security and Fuzz Testing

- Target: Secrets decryption, config parsing, memory safety
- Tools: `cargo-fuzz`
- Prevent crashes, memory bugs, and secret leakage
- **Security Note:** Decrypted secrets are only present in memory or at their destination after `ordinator apply`. They are never stored in the repository.

---

## 5. Test Environment

- Target OS: macOS (development and CI)
- Tests run in sandboxed temp environments
- Git, SOPS, and `age` must be available or mocked
- CI: GitHub Actions (or similar) with Rust toolchain, SOPS binary, SSH agent (if needed)

---

## 6. Responsibilities

| Role             | Responsibility                                      |
|------------------|----------------------------------------------------|
| Developer        | Write and maintain unit, integration, and CLI tests |
| Security Auditor | Conduct code reviews and security testing          |
| QA Engineer      | Validate test coverage and perform exploratory testing |

---

## 7. CI/CD Integration

- CI runs `cargo test`, linting (`clippy`), formatting (`rustfmt`), and fuzz targets
- PRs require all tests to pass before merging
- Coverage reporting via `cargo tarpaulin`
- Optional: Nightly fuzzing jobs

---

## 8. Metrics and Reporting

- Code coverage percentage via `tarpaulin`
- Track failed tests, flakes, and time-to-fix
- Document and triage any security-related failures
- Export JSON test logs if needed for CI dashboards

---

## 9. Risks and Mitigations

| Risk                                      | Mitigation                                        |
|-------------------------------------------|--------------------------------------------------|
| Secrets accidentally logged in plaintext | Strict logging and validation; plaintext scanner |
| System scripts mis-executed               | Never run sudo commands directly; generate reviewable shell script |
| Flaky tests or non-deterministic behavior | Use stable mocks, isolate side effects, and run tests in clean temp dirs |

---

## 10. Future Work

- Add test matrix for multiple macOS versions
- Consider Nix or Docker-based isolation for deterministic environments
- Expand integration test coverage to all CLI flags and edge cases 