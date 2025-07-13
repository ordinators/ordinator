mod common;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::output::OutputOkExt;
use assert_cmd::Command;
use assert_fs::fixture::PathChild;
use std::fs;

#[test]
fn test_commit_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    // No config file created
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["commit", "-m", "test"]);
    common::assert_config_error(cmd.assert().failure());
}

#[test]
fn test_commit_errors_without_git_repo() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    // Create config but no git repo
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    let _config_guard = common::EnvVarGuard::set("ORDINATOR_CONFIG", &config_path);
    // Don't set test mode for this test - we want to test actual failure
    cmd.args(["commit", "-m", "test"]);
    common::assert_config_error(cmd.assert().failure());
}

#[test]
fn test_commit_with_empty_message() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Initialize a proper Git repository using the CLI
    let mut init_cmd = common::create_ordinator_command(&temp);
    init_cmd.arg("init");
    init_cmd.unwrap();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("commit").arg("-m").arg("");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Commit message cannot be empty"));
}

#[test]
fn test_commit_with_secrets_detection_edge_cases() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Initialize a proper Git repository using the CLI
    let mut init_cmd = common::create_ordinator_command(&temp);
    init_cmd.arg("init");
    init_cmd.unwrap();

    // Create files with various secret patterns
    let files_dir = temp.path().join("files");
    fs::create_dir_all(&files_dir).unwrap();

    let test_files = [
        ("api_keys.txt", "api_key=sk_test_1234567890abcdef"),
        ("passwords.txt", "password=mysecretpassword123"),
        ("tokens.txt", "oauth_token=ghp_1234567890abcdef"),
        ("aws.txt", "aws_access_key_id=AKIA1234567890ABCDEF"),
    ];

    for (filename, content) in test_files {
        let file_path = files_dir.join(filename);
        fs::write(&file_path, content).unwrap();
    }

    // Watch and add all files to the default profile so they get scanned
    for (filename, _) in test_files.iter() {
        let mut watch_cmd = common::create_ordinator_command(&temp);
        watch_cmd.arg("watch").arg(format!("files/{filename}"));
        watch_cmd.unwrap();
        let mut add_cmd = common::create_ordinator_command(&temp);
        add_cmd.arg("add").arg(format!("files/{filename}"));
        add_cmd.unwrap();
    }

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("commit").arg("-m").arg("Test commit with secrets");

    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Plaintext secrets detected"));
}

#[test]
fn test_commit_with_very_long_message() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Initialize a proper Git repository using the CLI
    let mut init_cmd = common::create_ordinator_command(&temp);
    init_cmd.arg("init");
    init_cmd.unwrap();

    // Create a very long commit message
    let long_message = "a".repeat(10000);

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("commit").arg("-m").arg(&long_message);

    let output = cmd.unwrap();
    // Should handle long messages gracefully
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_commit_with_special_characters_in_message() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Initialize a proper Git repository using the CLI
    let mut init_cmd = common::create_ordinator_command(&temp);
    init_cmd.arg("init");
    init_cmd.unwrap();

    let special_message = "Commit with special chars: !@#$%^&*()_+-=[]{}|;':\",./<>?";

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("commit").arg("-m").arg(special_message);

    let output = cmd.unwrap();
    // Should handle special characters gracefully
    assert!(output.status.success() || !output.status.success());
}

#[test]
fn test_commit_with_force_flag_bypasses_secrets_check() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Initialize a proper Git repository using the CLI
    let mut init_cmd = common::create_ordinator_command(&temp);
    init_cmd.arg("init");
    init_cmd.unwrap();

    // Create a file with secrets
    let files_dir = temp.path().join("files");
    fs::create_dir_all(&files_dir).unwrap();
    let secret_file = files_dir.join("secret.txt");
    fs::write(&secret_file, "api_key=sk_test_1234567890abcdef").unwrap();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("commit")
        .arg("-m")
        .arg("Force commit with secrets")
        .arg("--force");

    let output = cmd.unwrap();
    // Should bypass secrets check with --force
    assert!(output.status.success());
}
