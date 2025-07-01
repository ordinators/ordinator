use assert_cmd::Command;
use predicates::str::contains;
use assert_fs::prelude::*;
use std::fs;

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
    cmd.assert().success().stdout(predicates::str::contains("DRY-RUN: Would add 'testfile.txt' to profile 'default'"));
}

#[test]
fn test_add_file_to_default_profile() {
    use assert_fs::prelude::*;
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Run ordinator init in temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["init"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Create the file to add in the same temp dir
    temp.child("testfile.txt").touch().unwrap();

    // Run ordinator add in the same temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "testfile.txt"]);
    cmd.assert().success().stdout(predicates::str::contains("Added 'testfile.txt' to profile 'default'"));

    // Check config file for tracked file string in the same temp dir
    assert!(config_path.exists(), "Config file does not exist at {:?}", config_path);
    let config_contents = fs::read_to_string(&config_path).unwrap();
    assert!(config_contents.contains("testfile.txt"));
}

#[test]
fn test_add_nonexistent_file_errors() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Run ordinator init
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["init"]);
    cmd.assert().success();

    // Try to add a file that does not exist in the same temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "does_not_exist.txt"]);
    cmd.assert().failure().stdout(predicates::str::contains("Path 'does_not_exist.txt' does not exist on disk."));
}

#[test]
fn test_add_nonexistent_directory_errors() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Run ordinator init in temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["init"]);
    cmd.assert().success();

    // Try to add a directory that does not exist in the same temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "no_such_dir/"]);
    cmd.assert().failure().stdout(predicates::str::contains("Path 'no_such_dir/' does not exist on disk."));
}

#[test]
fn test_add_to_nonexistent_profile_suggests_profile_add() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Run ordinator init in temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["init"]);
    cmd.assert().success();

    // Create the file to add in the same temp dir
    temp.child("testfile.txt").touch().unwrap();

    // Try to add a file to a non-existent profile in the same temp dir
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "testfile.txt", "--profile", "ghost"]);
    cmd.assert().failure().stdout(predicates::str::contains("Profile 'ghost' does not exist. To create it, run: ordinator profile add ghost"));
}

#[test]
fn test_add_file_excluded_by_global_pattern() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Write a config with a global exclude pattern
    std::fs::write(&config_path, r#"
[global]
default_profile = "default"
exclude = ["*.bak"]
[profiles.default]
files = []
directories = []
exclude = []
"#).unwrap();

    // Create a file that matches the global exclude pattern
    temp.child("secret.bak").touch().unwrap();

    // Try to add the excluded file
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "secret.bak"]);
    cmd.assert().failure().stdout(contains("matches an exclusion pattern and cannot be tracked"));
}

#[test]
fn test_add_file_excluded_by_profile_pattern() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Write a config with a profile-specific exclude pattern
    std::fs::write(&config_path, r#"
[global]
default_profile = "default"
exclude = []
[profiles.default]
files = []
directories = []
exclude = ["*.tmp"]
"#).unwrap();

    // Create a file that matches the profile exclude pattern
    temp.child("should_not_add.tmp").touch().unwrap();

    // Try to add the excluded file
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "should_not_add.tmp"]);
    cmd.assert().failure().stdout(contains("matches an exclusion pattern and cannot be tracked"));
}
