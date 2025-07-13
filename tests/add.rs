mod common;
use assert_cmd::prelude::*;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::*;
use predicates::str::contains;

#[test]
fn test_add_dry_run() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a test file
    temp.child("testfile.txt").write_str("content").unwrap();

    // First watch the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "testfile.txt"]);
    watch_cmd.assert().success();

    // Run add with dry-run flag
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "testfile.txt", "--dry-run"]);
    cmd.assert().success().stdout(contains(
        "Would update 'testfile.txt' for profile 'default'",
    ));
}

#[test]
fn test_add_file_to_default_profile() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    println!(
        "[DEBUG] Running test_add_file_to_default_profile in temp dir: {:?}",
        temp.path()
    );

    // Create the file to add in the same temp dir
    temp.child("testfile.txt").touch().unwrap();

    // First watch the file, then add it
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "testfile.txt"]);
    let watch_assert = watch_cmd.assert();
    let watch_output = watch_assert.get_output();
    println!("[DEBUG] ordinator watch status: {:?}", watch_output.status);
    println!(
        "[DEBUG] ordinator watch stdout: {}",
        String::from_utf8_lossy(&watch_output.stdout)
    );
    println!(
        "[DEBUG] ordinator watch stderr: {}",
        String::from_utf8_lossy(&watch_output.stderr)
    );
    watch_assert.success();

    // Now add the watched file
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "testfile.txt"]);
    let assert = cmd.assert();
    // Print output for debugging
    let output = assert.get_output();
    println!("[DEBUG] ordinator add status: {:?}", output.status);
    println!(
        "[DEBUG] ordinator add stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "[DEBUG] ordinator add stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert
        .success()
        .stdout(contains("Updated 'testfile.txt' for profile 'default'"));

    // Check config file for tracked file string in the same temp dir
    let config_file = temp.child("ordinator.toml");
    assert!(config_file.path().exists(), "Config file does not exist");
    let config_contents = std::fs::read_to_string(config_file.path()).unwrap();
    println!("[DEBUG] Config after add:\n{config_contents}");
    assert!(config_contents.contains("testfile.txt"));
}

#[test]
fn test_add_nonexistent_file_errors() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Try to add a file that does not exist in the same temp dir
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "does_not_exist.txt"]);
    cmd.assert().failure().stdout(contains(
        "File 'does_not_exist.txt' is not tracked for profile 'default'. Use 'ordinator watch does_not_exist.txt --profile default' to start tracking it."
    ));
}

#[test]
fn test_add_nonexistent_directory_errors() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Try to add a directory that does not exist in the same temp dir
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "no_such_dir/"]);
    cmd.assert().failure().stdout(contains(
        "File 'no_such_dir/' is not tracked for profile 'default'. Use 'ordinator watch no_such_dir/ --profile default' to start tracking it."
    ));
}

#[test]
fn test_add_to_nonexistent_profile_suggests_profile_add() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create the file to add in the same temp dir
    temp.child("testfile.txt").touch().unwrap();

    // First watch the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "testfile.txt"]);
    watch_cmd.assert().success();

    // Try to add a file to a non-existent profile in the same temp dir
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "testfile.txt", "--profile", "ghost"]);
    cmd.assert().failure().stdout(predicates::str::contains(
        "Profile 'ghost' does not exist. To create it, run: ordinator profile add ghost",
    ));
}

#[test]
fn test_add_file_excluded_by_global_pattern() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");

    // Write a config with a global exclude pattern
    std::fs::write(
        config_file.path(),
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

    // Try to watch the excluded file (should fail)
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "secret.bak"]);
    watch_cmd.assert().failure().stdout(contains(
        "matches an exclusion pattern and cannot be tracked",
    ));
}

#[test]
fn test_add_file_excluded_by_profile_pattern() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");

    // Write a config with a profile-specific exclude pattern
    std::fs::write(
        config_file.path(),
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

    // Try to watch the excluded file (should fail)
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "should_not_add.tmp"]);
    watch_cmd.assert().failure().stdout(contains(
        "matches an exclusion pattern and cannot be tracked",
    ));
}
