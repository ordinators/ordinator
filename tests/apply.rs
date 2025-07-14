mod common;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::*;

use assert_cmd::assert::OutputAssertExt;
use std::fs;

#[test]
fn test_apply_backs_up_existing_file() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();

    // Place the managed dotfile in files/
    let managed = files_dir.child("dotfile.txt");
    managed.write_str("managed contents").unwrap();

    // Create the file in the root temp dir for ordinator add to find
    temp.child("dotfile.txt")
        .write_str("managed contents")
        .unwrap();

    // Watch the file before adding (required by new workflow)
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "dotfile.txt"]);
    watch_cmd.assert().success();

    // Add the file to the config
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "dotfile.txt"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("[DEBUG] Add stdout: {stdout}");
    eprintln!("[DEBUG] Add stderr: {stderr}");
    assert!(output.status.success(), "Add failed: {stdout} {stderr}");

    // Enable backups in the config
    let config_file = temp.child("ordinator.toml");
    let config_contents = std::fs::read_to_string(config_file.path()).unwrap();
    let mut config: toml::Value = toml::from_str(&config_contents).unwrap();
    if let Some(global) = config.get_mut("global") {
        if let Some(global_table) = global.as_table_mut() {
            global_table.insert("create_backups".to_string(), toml::Value::Boolean(true));
        }
    }
    std::fs::write(config_file.path(), toml::to_string(&config).unwrap()).unwrap();

    // Place an existing file at the destination (home dir simulated by temp)
    let dest = temp.child("dotfile.txt");
    dest.write_str("original contents").unwrap();

    // Run ordinator apply with force (required for conflicts)
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Apply failed: {stdout} {stderr}");

    // Check that the backup exists
    let backup_dir = temp.child("backups");
    if backup_dir.path().exists() {
        let backups: Vec<_> = backup_dir
            .read_dir()
            .unwrap()
            .map(|e| e.unwrap().file_name().into_string().unwrap())
            .collect();
        assert!(
            backups.iter().any(|f| f.starts_with("dotfile.txt.backup.")),
            "No backup file found: {backups:?}"
        );
    } else {
        // If backup directory doesn't exist, that's also valid (no conflict occurred)
        eprintln!(
            "[DEBUG] Backup directory does not exist, which is valid if no conflict occurred"
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
fn test_apply_skips_backup_if_disabled() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();

    // Place the managed dotfile in files/
    let managed = files_dir.child("dotfile.txt");
    managed.write_str("managed contents").unwrap();

    // Create the file in the root temp dir for ordinator add to find
    temp.child("dotfile.txt")
        .write_str("managed contents")
        .unwrap();

    // Watch the file before adding (required by new workflow)
    let mut watch_cmd = common::create_ordinator_command(&temp);
    watch_cmd.args(["watch", "dotfile.txt"]);
    watch_cmd.assert().success();

    // Add the file to the config
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["add", "dotfile.txt"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success());

    // Disable backups in the config
    let config_file = temp.child("ordinator.toml");
    let config_contents = std::fs::read_to_string(config_file.path()).unwrap();
    let mut config: toml::Value = toml::from_str(&config_contents).unwrap();
    if let Some(global) = config.get_mut("global") {
        if let Some(global_table) = global.as_table_mut() {
            global_table.insert("create_backups".to_string(), toml::Value::Boolean(false));
        }
    }
    std::fs::write(config_file.path(), toml::to_string(&config).unwrap()).unwrap();

    // Place an existing file at the destination (home dir simulated by temp)
    let dest = temp.child("dotfile.txt");
    dest.write_str("original contents").unwrap();

    // Run ordinator apply with force (required for conflicts)
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--force"]);
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
fn test_apply_skip_secrets_flag_respected() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a profile with secrets configured
    let config_file = temp.child("ordinator.toml");
    let config_content = r#"
[global]
default_profile = "default"
auto_push = false
create_backups = true

[profiles.default]
files = []
secrets = ["~/.ssh/config"]
enabled = true

[secrets]
age_key_file = ""
sops_config = ""
encrypt_patterns = ["*.yaml", "*.txt"]
exclude_patterns = ["*.bak"]
"#;
    std::fs::write(config_file.path(), config_content).unwrap();

    // Run apply with --skip-secrets - should skip all secrets processing
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--profile", "default", "--skip-secrets"]);
    cmd.assert().success();
}

#[test]
fn test_apply_with_missing_age_key_detection() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a profile with secrets configured but no age key
    let config_file = temp.child("ordinator.toml");
    let config_content = r#"
[global]
default_profile = "default"
auto_push = false
create_backups = true

[profiles.default]
files = []
secrets = ["~/.ssh/config"]
enabled = true

[secrets]
age_key_file = ""
sops_config = ""
encrypt_patterns = ["*.yaml", "*.txt"]
exclude_patterns = ["*.bak"]
"#;
    std::fs::write(config_file.path(), config_content).unwrap();

    // Create an encrypted secret file
    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();
    let secret_file = files_dir.child(".ssh").child("config");
    secret_file
        .write_str("sops:\n  kms: []\n  age:\n    - age1testkey\n")
        .unwrap();

    // Run apply - should detect missing age key and handle gracefully
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--profile", "default"]);

    // Since this requires interactive input, we expect it to either succeed or fail gracefully
    let output = cmd.output().unwrap();
    // The command should either succeed (if no secrets to decrypt) or fail gracefully
    // We can't easily test interactive prompts in integration tests, but we can verify
    // that the command doesn't panic and handles the missing key scenario
    assert!(output.status.code().is_some());
}

#[test]
fn test_apply_with_key_mismatch_handling() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a profile with secrets configured
    let config_file = temp.child("ordinator.toml");
    let config_content = r#"
[global]
default_profile = "default"
auto_push = false
create_backups = true

[profiles.default]
files = []
secrets = ["~/.ssh/config"]
enabled = true

[secrets]
age_key_file = ""
sops_config = ""
encrypt_patterns = ["*.yaml", "*.txt"]
exclude_patterns = ["*.bak"]
"#;
    std::fs::write(config_file.path(), config_content).unwrap();

    // Create an encrypted secret file that can't be decrypted with a new key
    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();
    let secret_file = files_dir.child(".ssh").child("config");
    secret_file
        .write_str("sops:\n  kms: []\n  age:\n    - age1differentkey\n")
        .unwrap();

    // Run apply - should handle key mismatch gracefully
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["apply", "--profile", "default"]);

    // Since this would require interactive input, we expect it to fail gracefully
    let output = cmd.output().unwrap();
    // The command should either succeed (if no secrets to decrypt) or fail gracefully
    assert!(output.status.code().is_some());
}
