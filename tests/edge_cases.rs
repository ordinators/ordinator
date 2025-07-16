mod common;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::Command;
use assert_fs::fixture::FileWriteStr;
use assert_fs::fixture::PathChild;
use assert_fs::fixture::PathCreateDir;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_cli_with_corrupted_config() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a corrupted config file
    let config_file = temp.child("ordinator.toml");
    config_file.write_str("invalid toml content [").unwrap();

    // Set environment variables
    let _config_guard = common::EnvVarGuard::set("ORDINATOR_CONFIG", config_file.path());
    let _home_guard = common::EnvVarGuard::set("ORDINATOR_HOME", temp.path());
    let _test_guard = common::EnvVarGuard::set("ORDINATOR_TEST_MODE", "1");

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("status");

    cmd.assert()
        .failure()
        .stderr(contains("Failed to parse config file"));
}

#[test]
fn test_cli_with_invalid_json() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create a config file with invalid JSON-like content
    let config_file = temp.child("ordinator.toml");
    config_file
        .write_str(
            r#"
[global
default_profile = "default"
"#,
        )
        .unwrap();

    // Set environment variables
    let _config_guard = common::EnvVarGuard::set("ORDINATOR_CONFIG", config_file.path());
    let _home_guard = common::EnvVarGuard::set("ORDINATOR_HOME", temp.path());
    let _test_guard = common::EnvVarGuard::set("ORDINATOR_TEST_MODE", "1");

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("status");

    cmd.assert()
        .failure()
        .stderr(contains("Failed to parse config file"));
}

#[test]
fn test_cli_with_missing_dependencies() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a dummy git that fails
    let bin_dir = temp.child("bin");
    bin_dir.create_dir_all().unwrap();
    let git_path = bin_dir.child("git");
    git_path.write_str("#!/bin/sh\nexit 1\n").unwrap();
    fs::set_permissions(git_path.path(), fs::Permissions::from_mode(0o755)).unwrap();

    // Set PATH to use our dummy git
    let _path_guard = common::EnvVarGuard::set("PATH", bin_dir.path());
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", bin_dir.path());
    cmd.arg("status");

    // In test mode, git operations are skipped, so it should succeed
    cmd.assert()
        .success()
        .stderr(contains("TEST MODE").or(contains("Git repository")));
}

#[test]
fn test_cli_with_invalid_environment_variables() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Set invalid environment variables - don't set ORDINATOR_CONFIG to test "no config" scenario
    let _test_guard = common::EnvVarGuard::set("ORDINATOR_TEST_MODE", "1");

    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env(
        "ORDINATOR_TEST_NAME",
        "test_cli_with_invalid_environment_variables",
    );
    // Explicitly unset ORDINATOR_CONFIG and ORDINATOR_HOME to ensure no config is found
    cmd.env_remove("ORDINATOR_CONFIG");
    cmd.env_remove("ORDINATOR_HOME");
    cmd.arg("status");

    // CLI fails when no config file is found
    cmd.assert()
        .failure()
        .stderr(contains("No configuration file found"));
}

#[test]
fn test_cli_with_empty_config_file() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create an empty config file
    let config_file = temp.child("ordinator.toml");
    config_file.write_str("").unwrap();

    let _config_guard = common::EnvVarGuard::set("ORDINATOR_CONFIG", config_file.path());
    let _home_guard = common::EnvVarGuard::set("ORDINATOR_HOME", temp.path());
    let _test_guard = common::EnvVarGuard::set("ORDINATOR_TEST_MODE", "1");

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("status");

    // Empty config now succeeds but shows no Git repository message
    cmd.assert()
        .success()
        .stderr(contains("No Git repository found"));
}

#[test]
fn test_cli_with_malformed_profile_config() {
    let temp = assert_fs::TempDir::new().unwrap();

    // Create config with malformed profile
    let config_content = r#"
[global]
default_profile = "default"
auto_push = false
create_backups = true
exclude = []

[profiles.default]
files = ["invalid_file_that_does_not_exist"]
directories = ["invalid_directory_that_does_not_exist"]
enabled = true
description = "Profile with invalid files"
exclude = []

[secrets]
encrypt_patterns = []
exclude_patterns = []
"#;

    let (_config_guard, _test_mode_guard) =
        common::setup_test_environment_with_config(&temp, Some(config_content));

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("status");

    // Should not crash, but may show warnings
    cmd.assert().success();
}

#[test]
fn test_cli_with_unicode_filenames() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create file with unicode name
    let unicode_file = temp.child("test-🚀-file.txt");
    unicode_file.write_str("unicode content").unwrap();

    // Watch the unicode file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "test-🚀-file.txt"]);
    watch_cmd.assert().success();

    // Add the unicode file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "test-🚀-file.txt"]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_very_long_filenames() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create file with very long name
    let long_filename = "a".repeat(200) + ".txt";
    let long_file = temp.child(&long_filename);
    long_file.write_str("long filename content").unwrap();

    // Watch the long filename file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", &long_filename]);
    watch_cmd.assert().success();

    // Add the long filename file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", &long_filename]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_special_characters_in_filenames() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create file with special characters
    let special_filename = "test-file-with-special-chars-!@#$%^&*()_+-={}[]|\\:\";'<>?,./.txt";
    let special_file = temp.child(special_filename);
    special_file.write_str("special chars content").unwrap();

    // Watch the special filename file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", special_filename]);
    watch_cmd.assert().success();

    // Add the special filename file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", special_filename]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_nested_directories() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create nested directory structure
    let nested_dir = temp.child("nested").child("deep").child("directory");
    nested_dir.create_dir_all().unwrap();
    let nested_file = nested_dir.child("config.txt");
    nested_file.write_str("nested content").unwrap();

    // Watch the nested directory
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "nested/"]);
    watch_cmd.assert().success();

    // Add the nested directory - should fail because CLI only supports files, not directories
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "nested/"]);
    cmd.assert().failure().stderr(contains(
        "neither a regular file nor a symlink to a regular file",
    ));
}

#[test]
fn test_cli_with_symlinks_in_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a file and a symlink to it
    let original_file = temp.child("original.txt");
    original_file.write_str("original content").unwrap();

    let symlink_file = temp.child("symlink.txt");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(original_file.path(), symlink_file.path()).unwrap();
    }

    // Watch the symlink
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "symlink.txt"]);
    watch_cmd.assert().success();

    // Add the symlink
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "symlink.txt"]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_empty_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create empty file
    let empty_file = temp.child("empty.txt");
    empty_file.write_str("").unwrap();

    // Watch the empty file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "empty.txt"]);
    watch_cmd.assert().success();

    // Add the empty file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "empty.txt"]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_large_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a large file (1MB)
    let large_file = temp.child("large.txt");
    let large_content = "x".repeat(1024 * 1024);
    large_file.write_str(&large_content).unwrap();

    // Watch the large file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "large.txt"]);
    watch_cmd.assert().success();

    // Add the large file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "large.txt"]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_binary_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a binary file
    let binary_file = temp.child("binary.bin");
    let binary_content = vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD];
    fs::write(binary_file.path(), binary_content).unwrap();

    // Watch the binary file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "binary.bin"]);
    watch_cmd.assert().success();

    // Add the binary file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "binary.bin"]);
    cmd.assert().success();
}

#[test]
fn test_cli_with_concurrent_access() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a file
    let test_file = temp.child("concurrent.txt");
    test_file.write_str("concurrent content").unwrap();

    // Watch the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "concurrent.txt"]);
    watch_cmd.assert().success();

    // Add the file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "concurrent.txt"]);
    cmd.assert().success();

    // Try to add the same file again - now updates the file instead of failing
    let mut cmd2 = common::create_ordinator_command(&temp);
    cmd2.args(["add", "concurrent.txt"]);
    cmd2.assert().success().stdout(contains("Updated"));
}

#[test]
fn test_cli_with_missing_watched_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Try to add a file that hasn't been watched
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "unwatched.txt"]);
    cmd.assert().failure().stderr(contains("not tracked"));
}

#[test]
fn test_cli_with_invalid_command_arguments() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Test various invalid command combinations
    let invalid_commands = vec![
        vec!["add", "--invalid-flag"],
        vec!["watch", "--nonexistent-option"],
        vec!["commit", "-m"],                // missing message
        vec!["init", "--repo", "--profile"], // missing values
    ];

    for args in invalid_commands {
        let mut cmd = common::create_ordinator_command(&temp);
        cmd.args(&args);
        cmd.assert().failure();
    }

    // Test that some combinations that were previously considered invalid are now valid
    let valid_commands = vec![
        vec!["apply", "--force", "--dry-run"], // conflicting flags are now accepted
    ];

    for args in valid_commands {
        let mut cmd = common::create_ordinator_command(&temp);
        cmd.args(&args);
        cmd.assert().success();
    }
}

#[test]
fn test_cli_with_network_failures() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) =
        common::setup_test_environment_with_config_and_test_name(
            &temp,
            None,
            Some("test_cli_with_network_failures"),
        );

    // Assert the config comment matches the test name
    common::assert_config_comment_matches(&temp, "test_cli_with_network_failures");

    // Test with invalid remote URL - in test mode, we need to provide a target directory
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://invalid-url-that-will-fail.com/user/repo.git",
        temp.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .failure()
        .stderr(contains("Invalid GitHub URL"));
}

#[test]
fn test_cli_with_temporary_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a temporary file
    let temp_file = temp.child("temp.txt");
    temp_file.write_str("temporary content").unwrap();

    // Watch the temporary file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "temp.txt"]);
    watch_cmd.assert().success();

    // Add the temporary file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "temp.txt"]);
    cmd.assert().success();

    // Remove the file and try to apply
    fs::remove_file(temp_file.path()).unwrap();
    let mut apply_cmd = common::create_ordinator_command(&temp);
    apply_cmd.args(["apply", "--force"]);
    apply_cmd.assert().success();
}
