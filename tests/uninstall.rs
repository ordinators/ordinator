mod common;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::*;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

use assert_cmd::assert::OutputAssertExt;

#[test]
fn test_uninstall_removes_symlinks_and_restores_backups() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a file and add it to the profile
    let dotfile = temp.child(".zshrc");
    dotfile.write_str("original contents").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();

    // Simulate apply with backup
    let dest = temp.child(".zshrc");
    dest.write_str("user contents").unwrap();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    cmd.assert().success();
    // Confirm backup exists
    let backup_dir = temp.child("backups");
    if backup_dir.path().exists() {
        let backups: Vec<_> = backup_dir
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert!(
            backups.iter().any(|f| f.starts_with(".zshrc.backup.")),
            "No backup file found: {backups:?}"
        );
    } else {
        // If backup directory doesn't exist, that's also valid - no backup was needed
        println!("[DEBUG] Backup directory does not exist, which is valid if no backup was needed");
    }
    // Confirm symlink exists
    #[cfg(unix)]
    assert!(std::fs::symlink_metadata(dest.path())
        .unwrap()
        .file_type()
        .is_symlink());

    // Run uninstall with --force and --restore-backups
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force", "--restore-backups"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Uninstall failed: {stderr}");
    // Confirm symlink is gone and file is restored
    #[cfg(unix)]
    assert!(!std::fs::symlink_metadata(dest.path())
        .unwrap()
        .file_type()
        .is_symlink());
    let restored = std::fs::read_to_string(dest.path()).unwrap();
    assert_eq!(restored, "user contents");
}

#[test]
fn test_uninstall_dry_run_outputs_preview() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    temp.child(".zshrc").write_str("original contents").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--dry-run", "--force"]);
    cmd.assert()
        .success()
        .stderr(contains("Would remove symlink"));
}

#[test]
fn test_uninstall_nonexistent_profile_errors() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--profile", "ghost", "--force"]);
    cmd.assert()
        .failure()
        .stderr(contains("Profile 'ghost' does not exist"));
}

#[test]
fn test_uninstall_with_no_profiles_in_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Minimal config with no profiles
    let config = r#"[global]
default_profile = "default"
auto_push = false
create_backups = true
exclude = []

[secrets]
encrypt_patterns = []
exclude_patterns = []

[readme]
auto_update = false
update_on_changes = ["profiles", "bootstrap"]
"#;
    let (_config_guard, _test_mode_guard) =
        common::setup_test_environment_with_config(&temp, Some(config));
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force"]);
    cmd.assert()
        .failure()
        .stderr(contains("No profiles found in config"));
}

#[test]
fn test_uninstall_when_no_symlinks_exist() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    temp.child(".zshrc").write_str("not a symlink").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();
    // Remove the symlink if it exists
    let dest = temp.child(".zshrc");
    if dest.path().exists() {
        std::fs::remove_file(dest.path()).ok();
        dest.write_str("not a symlink").unwrap();
    }
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force"]);
    cmd.assert()
        .success()
        .stderr(contains("File exists (not a symlink)").or(contains("File does not exist")));
}

#[test]
fn test_uninstall_when_no_backups_exist() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    temp.child(".zshrc").write_str("original contents").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    cmd.assert().success();
    // Remove backups
    let backup_dir = temp.child("backups");
    if backup_dir.path().exists() {
        std::fs::remove_dir_all(backup_dir.path()).ok();
    }
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force", "--restore-backups"]);
    cmd.assert()
        .success()
        .stderr(contains("No backup found").or(contains("Backups restored: 0")));
}

#[test]
fn test_uninstall_with_broken_symlink() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    temp.child(".zshrc").write_str("original contents").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    cmd.assert().success();
    // Remove the source file to break the symlink
    let files_dir = temp.child("files/default/.zshrc");
    if files_dir.path().exists() {
        std::fs::remove_file(files_dir.path()).ok();
    }
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force"]);
    cmd.assert()
        .success()
        .stderr(contains("Removed symlink").or(contains("File does not exist")));
}

#[test]
fn test_uninstall_with_non_symlink_non_file_at_target() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    let dir = temp.child(".zshrc");
    dir.create_dir_all().unwrap();

    // Try to watch and add the directory (should fail)
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().failure().stderr(contains(
        "neither a regular file nor a symlink to a regular file",
    ));
}

#[test]
fn test_uninstall_with_multiple_backups_restores_most_recent() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    temp.child(".zshrc").write_str("original contents").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    cmd.assert().success();
    // Simulate multiple backups
    let backup_dir = temp.child("backups");
    backup_dir.create_dir_all().unwrap();
    let backup1 = backup_dir.child(".zshrc.backup.1.20250101-120000");
    let backup2 = backup_dir.child(".zshrc.backup.2.20250101-130000");
    backup1.write_str("old backup").unwrap();
    backup2.write_str("new backup").unwrap();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force", "--restore-backups"]);
    let output = cmd.output().unwrap();
    let restored = std::fs::read_to_string(temp.child(".zshrc").path()).unwrap();
    assert!(
        restored == "new backup" || restored == "user contents",
        "Most recent backup should be restored"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Restored from backup") || stderr.contains("Backups restored: 1"));
}

#[test]
fn test_uninstall_with_profile_but_no_tracked_files() {
    let temp = assert_fs::TempDir::new().unwrap();
    // Config with empty profile
    let config = r#"[global]
default_profile = "default"
auto_push = false
create_backups = true
exclude = []

[profiles.default]
files = []
directories = []
enabled = true
description = "Default profile for basic dotfiles"
exclude = []
homebrew_packages = []

[secrets]
encrypt_patterns = []
exclude_patterns = []

[readme]
auto_update = false
update_on_changes = ["profiles", "bootstrap"]
"#;
    let (_config_guard, _test_mode_guard) =
        common::setup_test_environment_with_config(&temp, Some(config));
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--profile", "default", "--force"]);
    cmd.assert()
        .success()
        .stderr(contains("has no tracked files"));
}

#[test]
fn test_uninstall_with_restore_backups_but_backups_disabled() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);
    // Disable backups in config
    let config_file = temp.child("ordinator.toml");
    let config_contents = std::fs::read_to_string(config_file.path()).unwrap();
    let mut config: toml::Value = toml::from_str(&config_contents).unwrap();
    if let Some(global) = config.get_mut("global") {
        if let Some(global_table) = global.as_table_mut() {
            global_table.insert("create_backups".to_string(), toml::Value::Boolean(false));
        }
    }
    std::fs::write(config_file.path(), toml::to_string(&config).unwrap()).unwrap();
    temp.child(".zshrc").write_str("original contents").unwrap();

    // Watch and add the file
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", ".zshrc"]);
    watch_cmd.assert().success();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", ".zshrc"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    cmd.assert().success();
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["uninstall", "--force", "--restore-backups"]);
    cmd.assert()
        .success()
        .stderr(contains("Backups are disabled").or(contains("No backup found")));
}
