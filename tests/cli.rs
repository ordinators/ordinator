use assert_cmd::Command;
use assert_fs::fixture::PathChild;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn assert_config_error(assert: assert_cmd::assert::Assert) -> assert_cmd::assert::Assert {
    assert
        .stderr(contains("No configuration file found").or(contains("Failed to parse config file")))
}

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
    cmd.assert().success().stdout(predicates::str::contains(
        "DRY-RUN: Would add 'testfile.txt' to profile 'default'",
    ));
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
    cmd.assert().success().stdout(predicates::str::contains(
        "Added 'testfile.txt' to profile 'default'",
    ));

    // Check config file for tracked file string in the same temp dir
    assert!(
        config_path.exists(),
        "Config file does not exist at {config_path:?}"
    );
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
    cmd.assert().failure().stdout(predicates::str::contains(
        "Path 'does_not_exist.txt' does not exist on disk.",
    ));
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
    cmd.assert().failure().stdout(predicates::str::contains(
        "Path 'no_such_dir/' does not exist on disk.",
    ));
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
    cmd.assert().failure().stdout(predicates::str::contains(
        "Profile 'ghost' does not exist. To create it, run: ordinator profile add ghost",
    ));
}

#[test]
fn test_add_file_excluded_by_global_pattern() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Write a config with a global exclude pattern
    std::fs::write(
        &config_path,
        r#"
[global]
default_profile = "default"
exclude = ["*.bak"]
[profiles.default]
files = []
directories = []
exclude = []
"#,
    )
    .unwrap();

    // Create a file that matches the global exclude pattern
    temp.child("secret.bak").touch().unwrap();

    // Try to add the excluded file
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "secret.bak"]);
    cmd.assert().failure().stdout(contains(
        "matches an exclusion pattern and cannot be tracked",
    ));
}

#[test]
fn test_add_file_excluded_by_profile_pattern() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();

    // Write a config with a profile-specific exclude pattern
    std::fs::write(
        &config_path,
        r#"
[global]
default_profile = "default"
exclude = []
[profiles.default]
files = []
directories = []
exclude = ["*.tmp"]
"#,
    )
    .unwrap();

    // Create a file that matches the profile exclude pattern
    temp.child("should_not_add.tmp").touch().unwrap();

    // Try to add the excluded file
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "should_not_add.tmp"]);
    cmd.assert().failure().stdout(contains(
        "matches an exclusion pattern and cannot be tracked",
    ));
}

#[test]
fn test_apply_backs_up_existing_file() {
    use assert_fs::prelude::*;
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();

    // Write a config with create_backups = true and track 'dotfile.txt'
    std::fs::write(
        &config_path,
        r#"
[global]
default_profile = "default"
create_backups = true
[profiles.default]
files = ["dotfile.txt"]
directories = []
exclude = []
"#,
    )
    .unwrap();

    // Place the managed dotfile in files/
    let managed = files_dir.child("dotfile.txt");
    managed.write_str("managed contents").unwrap();

    // Place an existing file at the destination (home dir simulated by temp)
    let dest = temp.child("dotfile.txt");
    dest.write_str("original contents").unwrap();

    // Run ordinator apply
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["apply"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Apply failed: {stdout} {stderr}");
    // Check that the backup exists
    let backup_dir = temp.child("backups");
    let backups: Vec<_> = backup_dir
        .read_dir()
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert!(
        backups.iter().any(|f| f.starts_with("dotfile.txt-")),
        "No backup file found: {backups:?}"
    );
    // Check that the destination is now a symlink
    #[cfg(unix)]
    {
        let meta = fs::symlink_metadata(dest.path()).unwrap();
        assert!(
            meta.file_type().is_symlink(),
            "Destination is not a symlink"
        );
    }
}

#[test]
fn test_apply_skips_backup_if_disabled() {
    use assert_fs::prelude::*;
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();

    // Write a config with create_backups = false and track 'dotfile.txt'
    std::fs::write(
        &config_path,
        r#"
[global]
default_profile = "default"
create_backups = false
[profiles.default]
files = ["dotfile.txt"]
directories = []
exclude = []
"#,
    )
    .unwrap();

    // Place the managed dotfile in files/
    let managed = files_dir.child("dotfile.txt");
    managed.write_str("managed contents").unwrap();

    // Place an existing file at the destination (home dir simulated by temp)
    let dest = temp.child("dotfile.txt");
    dest.write_str("original contents").unwrap();

    // Run ordinator apply
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["apply"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Apply failed: {stdout} {stderr}");
    // Check that the backup directory does not exist or is empty
    let backup_dir = temp.child("backups");
    if backup_dir.path().exists() {
        let backups: Vec<_> = backup_dir
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert!(
            backups.is_empty(),
            "Backup directory should be empty, found: {backups:?}"
        );
    }
    // Check that the destination is now a symlink
    #[cfg(unix)]
    {
        let meta = fs::symlink_metadata(dest.path()).unwrap();
        assert!(
            meta.file_type().is_symlink(),
            "Destination is not a symlink"
        );
    }
}

#[test]
fn test_commit_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    // No config file created
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["commit", "-m", "test"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_commit_errors_without_git_repo() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    // Create config but no git repo
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["commit", "-m", "test"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_push_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["push"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_push_errors_without_git_repo() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["push"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_pull_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["pull"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_pull_errors_without_git_repo() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["pull"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_sync_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["sync"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_sync_errors_without_git_repo() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["sync"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_status_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["status"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_status_errors_without_git_repo() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["status"]);
    assert_config_error(cmd.assert().failure());
}

#[test]
fn test_profiles_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["profiles"]);
    assert_config_error(cmd.assert().failure());
}
