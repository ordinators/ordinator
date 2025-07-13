mod common;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::*;

use assert_cmd::assert::OutputAssertExt;
use std::fs;

#[test]
fn test_status_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = assert_cmd::Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["status"]);
    common::assert_config_error(cmd.assert().failure());
}

#[test]
fn test_status_symlinks_and_repair() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();
    let managed = files_dir.child("status_test_file.txt");
    managed.write_str("hello").unwrap();

    // Create the file in the root temp dir for ordinator add to find
    temp.child("status_test_file.txt")
        .write_str("hello")
        .unwrap();

    // Watch and add the file to the config
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "status_test_file.txt"]);
    watch_cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "status_test_file.txt"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Remove the destination file if it exists
    let home_file = temp.child("status_test_file.txt");
    if home_file.path().exists() {
        fs::remove_file(home_file.path()).unwrap();
    }

    // First apply to create the symlink
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force", "--verbose"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("[DEBUG] Apply stdout: {stdout}");
    eprintln!("[DEBUG] Apply stderr: {stderr}");
    assert!(output.status.success(), "Initial apply failed");

    // Verify symlink was created
    let home_file = temp.child("status_test_file.txt");
    #[cfg(unix)]
    {
        let meta = fs::symlink_metadata(home_file.path()).unwrap();
        assert!(meta.file_type().is_symlink(), "Symlink was not created");
    }

    // Run status with verbose
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["status", "--verbose"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Status failed: {stderr}");
    assert!(
        stderr.contains("Valid symlink"),
        "Expected 'Valid symlink' in stderr, got: {stderr}"
    );

    // Break the symlink by removing the target
    fs::remove_file(managed.path()).unwrap();

    // Run repair
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["repair", "--verbose"]);
    let output = cmd.output().unwrap();
    let _stdout = String::from_utf8_lossy(&output.stdout);
    let _stderr = String::from_utf8_lossy(&output.stderr);
    // The repair command should remove the broken symlink
    assert!(!home_file.path().exists(), "Broken symlink was not removed");
}
