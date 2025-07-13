mod common;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::fixture::FileWriteStr;
use assert_fs::fixture::PathChild;
use predicates::str::contains;

#[test]
fn test_bootstrap_command_success() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Run bootstrap command
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "work"]);
    let assert = cmd.assert();

    // Should show script info and safety level
    assert
        .success()
        .stderr(contains("Bootstrap script info for profile: work"))
        .stderr(contains("bootstrap.sh"))
        .stderr(contains("Safety level:"))
        .stderr(contains("To run the bootstrap script"));
}

#[test]
fn test_bootstrap_command_nonexistent_profile() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Run bootstrap command with non-existent profile
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "nonexistent"]);
    let assert = cmd.assert();

    // Should fail with profile not found error
    assert
        .failure()
        .stderr(contains("Profile 'nonexistent' does not exist"));
}

#[test]
fn test_bootstrap_command_no_script_defined() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Run bootstrap command with profile that has no bootstrap script
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "default"]);
    let assert = cmd.assert();

    // Should show no script defined message
    assert.success().stderr(contains(
        "No bootstrap script defined for profile 'default'",
    ));
}

#[test]
fn test_bootstrap_command_dry_run() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Run bootstrap command with dry-run
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "work", "--dry-run"]);
    let assert = cmd.assert();

    // Should show DRY-RUN message
    assert
        .success()
        .stderr(contains("DRY-RUN"))
        .stderr(contains("Bootstrap script info for profile: work"));
}

#[test]
fn test_bootstrap_command_quiet_mode() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Run bootstrap command with quiet flag
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "work", "--quiet"]);
    let assert = cmd.assert();

    // Should not show the info messages but still show the command to run
    assert
        .success()
        .stderr(contains("To run the bootstrap script"))
        .stderr(contains("bash"));
}

#[test]
fn test_bootstrap_command_with_edit_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Mock EDITOR environment variable
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("EDITOR", "echo"); // Use echo as a mock editor
    cmd.args(["bootstrap", "--profile", "work", "--edit"]);
    let assert = cmd.assert();

    // Should show script opened for editing
    assert
        .success()
        .stderr(contains("Script opened for editing"));
}

#[test]
fn test_bootstrap_integration_with_apply() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Run apply command which should generate bootstrap script
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--profile", "work"]);
    let assert = cmd.assert();

    // Should show bootstrap script generation
    assert
        .success()
        .stderr(contains("Generated bootstrap script"))
        .stderr(contains("bootstrap.sh"));
}

#[test]
fn test_bootstrap_script_safety_levels() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Create a dangerous bootstrap script
    let dangerous_script = r#"#!/bin/bash
sudo apt update
echo "Dangerous script"
"#;
    temp.child("bootstrap.sh")
        .write_str(dangerous_script)
        .unwrap();

    // Run bootstrap command
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "work"]);
    let assert = cmd.assert();

    // Should show dangerous warning
    assert
        .success()
        .stderr(contains("DANGEROUS"))
        .stderr(contains("sudo"));
}

#[test]
fn test_bootstrap_script_blocked_level() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a config with a bootstrap script defined
    let config_content = r#"
[profiles.work]
bootstrap_script = "bootstrap.sh"
files = []
"#;
    temp.child("ordinator.toml")
        .write_str(config_content)
        .unwrap();

    // Create a blocked bootstrap script
    let blocked_script = r#"#!/bin/bash
rm -rf /
echo "Blocked script"
"#;
    temp.child("bootstrap.sh")
        .write_str(blocked_script)
        .unwrap();

    // Run bootstrap command
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["bootstrap", "--profile", "work"]);
    let assert = cmd.assert();

    // Should show blocked warning
    assert
        .success()
        .stderr(contains("BLOCKED"))
        .stderr(contains("rm -rf /"));
}
