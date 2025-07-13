mod common;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::Command;
use assert_fs::fixture::FileTouch;
use assert_fs::fixture::PathChild;
use predicates::str::contains;

#[test]
fn test_profiles_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["profiles"]);
    common::assert_config_error(cmd.assert().failure());
}

#[test]
fn test_profiles_success() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("profiles");

    cmd.assert()
        .success()
        .stderr(contains("default"))
        .stderr(contains("work"))
        .stderr(contains("personal"));
}

#[test]
fn test_profiles_verbose() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("profiles").arg("--verbose");

    cmd.assert()
        .success()
        .stderr(contains("Default profile for basic dotfiles"))
        .stderr(contains("Work environment profile"))
        .stderr(contains("Personal environment profile"));
}

#[test]
fn test_profiles_with_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Add some files to make profiles more interesting
    let test_file = temp.child("test.txt");
    test_file.touch().unwrap();

    // Watch and add the file to work profile
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "test.txt", "--profile", "work"]);
    watch_cmd.assert().success();

    let mut add_cmd = common::create_ordinator_command(&temp);
    add_cmd
        .arg("add")
        .arg("test.txt")
        .arg("--profile")
        .arg("work");
    add_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.arg("profiles").arg("--verbose");

    cmd.assert()
        .success()
        .stderr(contains("work"))
        .stderr(contains("Work environment profile"));
}
