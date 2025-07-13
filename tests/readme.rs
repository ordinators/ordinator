mod common;
use assert_fs::fixture::PathChild;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn test_readme_default_success() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Run the ordinator readme default command
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["readme", "default"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Readme default failed: {stderr}");
    assert!(
        stdout.contains("README") || stderr.contains("README"),
        "Expected README message"
    );

    // Check that README.md file was created
    let readme_file = temp.child("README.md");
    assert!(
        readme_file.path().exists(),
        "README.md file was not created"
    );
}

#[test]
fn test_readme_default_without_config_fails() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Don't set ORDINATOR_CONFIG environment variable to simulate no config
    let mut cmd = assert_cmd::Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    // Explicitly unset ORDINATOR_CONFIG to ensure no config is found
    cmd.env_remove("ORDINATOR_CONFIG");
    cmd.args(["readme", "default"]);
    cmd.assert()
        .failure()
        .stderr(contains("No configuration file found").or(contains("error")));
}

#[test]
fn test_readme_help_command() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["readme", "--help"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("default") && stdout.contains("README"),
        "Expected help output for readme command"
    );
}
