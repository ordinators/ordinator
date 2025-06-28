use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(contains("Dotfiles and Environment Manager for macOS"))
        .stdout(contains("Usage:"))
        .stdout(contains("init"))
        .stdout(contains("add"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.arg("--version");
    cmd.assert().success().stdout(contains("ordinator "));
}

#[test]
fn test_init_dry_run() {
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.args(["init", "--dry-run"]);
    cmd.assert()
        .success()
        .stderr(contains("DRY-RUN"))
        .stderr(contains("Initializing repository"));
}

#[test]
fn test_add_dry_run() {
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.args(["add", "testfile.txt", "--dry-run"]);
    cmd.assert()
        .success()
        .stderr(contains("DRY-RUN"))
        .stderr(contains("Adding file"));
}
