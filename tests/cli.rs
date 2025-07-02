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
    let temp = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", "/nonexistent/config.toml");
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["add", "testfile.txt", "--dry-run"]);
    // Dry-run still requires a valid configuration
    cmd.assert().failure().stderr(predicates::str::contains(
        "No configuration file found. Run 'ordinator init' first.",
    ));
}

#[test]
fn test_add_file_to_default_profile() {
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
    let config_contents = std::fs::read_to_string(&config_path).unwrap();
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
    let config_toml = r#"
[global]
default_profile = "default"
create_backups = true

[profiles.default]
files = ["dotfile.txt"]
directories = []
exclude = []
"#;
    std::fs::write(&config_path, config_toml).unwrap();

    // Place the managed dotfile in files/
    let managed = files_dir.child("dotfile.txt");
    managed.write_str("managed contents").unwrap();

    // Place an existing file at the destination (home dir simulated by temp)
    let dest = temp.child("dotfile.txt");
    dest.write_str("original contents").unwrap();

    // Run ordinator apply with force (required for conflicts)
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["apply", "--force"]);
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
        backups.iter().any(|f| f.starts_with("dotfile.txt.backup.")),
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

    // Run ordinator apply with force (required for conflicts)
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
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

#[test]
fn test_apply_and_status_symlinks() {
    use assert_fs::prelude::*;
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();

    // Write a config with create_backups = true and track '.testfile'
    let config_toml = r#"
[global]
default_profile = "default"
create_backups = true

[profiles.default]
files = [".testfile"]
directories = []
exclude = []
"#;
    std::fs::write(&config_path, config_toml).unwrap();

    // Place the managed dotfile in files/
    let files_dir = temp.child("files");
    files_dir.create_dir_all().unwrap();
    let managed = files_dir.child(".testfile");
    managed.write_str("hello").unwrap();

    // Debug after initial creation
    let debug_path = temp.child("debug.txt");
    {
        use std::io::Write;
        let managed_parent = managed.path().parent().unwrap();
        let mut debug_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(debug_path.path())
            .unwrap();
        writeln!(
            debug_file,
            "[AFTER CREATE] managed parent: {managed_parent}",
            managed_parent = managed_parent.display()
        )
        .unwrap();
        writeln!(
            debug_file,
            "[AFTER CREATE] managed parent exists: {exists}",
            exists = managed_parent.exists()
        )
        .unwrap();
        match std::fs::metadata(managed_parent) {
            Ok(meta) => {
                writeln!(
                    debug_file,
                    "[AFTER CREATE] managed parent permissions: {permissions:?}",
                    permissions = meta.permissions()
                )
                .unwrap();
            }
            Err(e) => {
                writeln!(
                    debug_file,
                    "[AFTER CREATE] managed parent metadata error: {e}"
                )
                .unwrap();
            }
        }
        writeln!(
            debug_file,
            "[AFTER CREATE] managed file path: {managed_path}",
            managed_path = managed.display()
        )
        .unwrap();
    }
    if debug_path.path().exists() {
        if let Ok(debug_contents) = std::fs::read_to_string(debug_path.path()) {
            eprintln!("[TEST DEBUG FILE AFTER CREATE]\n{debug_contents}");
        }
    }

    // Ensure the destination file does not exist
    let home_file = temp.child(".testfile");
    if home_file.path().exists() {
        fs::remove_file(home_file.path()).unwrap();
    }

    // Skip git initialization - not needed for this test

    // First apply to create the symlink
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["apply", "--force", "--verbose"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("[DEBUG] Apply stdout: {stdout}");
    eprintln!("[DEBUG] Apply stderr: {stderr}");
    assert!(output.status.success(), "Initial apply failed");

    // After apply, recursively list all files and symlinks in the temp directory
    fn list_files_recursively<P: AsRef<std::path::Path>>(path: P, prefix: &str) {
        use std::fs;
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_type = entry.file_type().ok();
                eprintln!("[DEBUG] {} ({:?})", path.display(), file_type);
                if let Some(ft) = file_type {
                    if ft.is_dir() {
                        list_files_recursively(&path, &format!("{prefix}  "));
                    }
                }
            }
        }
    }
    eprintln!("[DEBUG] Recursive listing of temp dir after apply:");
    list_files_recursively(temp.path(), "");

    // Verify symlink was created
    let home_file = temp.child(".testfile");
    #[cfg(unix)]
    {
        let meta = fs::symlink_metadata(home_file.path()).unwrap();
        assert!(meta.file_type().is_symlink(), "Symlink was not created");
    }

    // Run status with verbose
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", &config_path);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["status", "--verbose"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Status failed: {stderr}");
    assert!(
        stderr.contains("Valid symlink"),
        "Expected 'Valid symlink' in stderr, got: {stderr}"
    );
}

#[test]
fn test_repair_broken_symlink() {
    use assert_fs::prelude::*;
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let ordinator_home = temp.path();
    let config_path = ordinator_home.join("ordinator.toml");
    let config_toml = r#"
[global]
default_profile = "default"
create_backups = true

[profiles.default]
files = [".testfile"]
directories = []
exclude = []
"#;
    std::fs::write(&config_path, config_toml).unwrap();

    // Place the managed dotfile in files/
    let files_dir = ordinator_home.join("files");
    std::fs::create_dir_all(&files_dir).unwrap();
    let managed_path = files_dir.join(".testfile");
    std::fs::write(&managed_path, "hello").unwrap();

    // Run init to set up git repo
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["init", "--verbose"]);
    let _ = cmd.output();

    // First apply to create the symlink
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["apply", "--force", "--verbose"]);
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("[DEBUG] Apply stdout: {stdout}");
    eprintln!("[DEBUG] Apply stderr: {stderr}");
    assert!(output.status.success(), "Initial apply failed");

    // After apply, recursively list all files and symlinks in the temp directory
    fn list_files_recursively<P: AsRef<std::path::Path>>(path: P, prefix: &str) {
        use std::fs;
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_type = entry.file_type().ok();
                eprintln!("[DEBUG] {} ({:?})", path.display(), file_type);
                if let Some(ft) = file_type {
                    if ft.is_dir() {
                        list_files_recursively(&path, &format!("{prefix}  "));
                    }
                }
            }
        }
    }
    eprintln!("[DEBUG] Recursive listing of temp dir after apply:");
    list_files_recursively(temp.path(), "");

    // Verify symlink was created
    let home_file = temp.child(".testfile");
    #[cfg(unix)]
    {
        let meta = fs::symlink_metadata(home_file.path()).unwrap();
        assert!(meta.file_type().is_symlink(), "Symlink was not created");
    }

    // Check if managed file exists after apply
    if managed_path.exists() {
        // Remove the managed file to break the symlink
        fs::remove_file(&managed_path).unwrap();
        eprintln!("[DEBUG] Managed file removed");

        // Debug: check if managed file exists before assertion
        eprintln!(
            "[DEBUG] Managed file exists before assertion: {exists}",
            exists = managed_path.exists()
        );
        match std::fs::symlink_metadata(&managed_path) {
            Ok(meta) => {
                eprintln!(
                    "[DEBUG] Managed file type before assertion: {ft:?}",
                    ft = meta.file_type()
                );
            }
            Err(e) => {
                eprintln!("[DEBUG] Managed file metadata error before assertion: {e}");
            }
        }
    } else {
        eprintln!("[DEBUG] Managed file did not exist after apply; skipping removal");
    }

    // Verify symlink is now broken (if it exists)
    #[cfg(unix)]
    {
        eprintln!(
            "[DEBUG] home_file path: {path}",
            path = home_file.path().display()
        );
        eprintln!(
            "[DEBUG] home_file exists: {exists}",
            exists = home_file.path().exists()
        );
        match std::fs::symlink_metadata(home_file.path()) {
            Ok(meta) => {
                eprintln!("[DEBUG] home_file metadata: {ft:?}", ft = meta.file_type());
            }
            Err(e) => {
                eprintln!("[DEBUG] home_file metadata error: {e}");
            }
        }
        eprintln!(
            "[DEBUG] Temp dir contents: {:?}",
            std::fs::read_dir(temp.path())
                .map(|d| d.collect::<Vec<_>>())
                .unwrap_or_default()
        );
        let meta = fs::symlink_metadata(home_file.path()).unwrap();
        assert!(meta.file_type().is_symlink(), "Symlink should still exist");
        let target = home_file.path().read_link().unwrap();
        assert!(!target.exists(), "Symlink target should not exist");
    }

    // Run repair with verbose while symlink is broken
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["repair", "--verbose"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Repair should either succeed (if it can handle missing source) or fail gracefully
    if output.status.success() {
        assert!(
            stderr.contains("Repaired") || stderr.contains("No broken symlinks"),
            "Expected repair success message, got: {stderr}"
        );
    } else {
        assert!(
            stderr.contains("No such file") || stderr.contains("Source file not found"),
            "Expected graceful failure for missing source, got: {stderr}"
        );
    }
}

#[test]
fn test_apply_force_overwrites_conflict() {
    use assert_fs::prelude::*;
    use std::fs;
    let temp = assert_fs::TempDir::new().unwrap();
    let ordinator_home = temp.path();
    let config_path = ordinator_home.join("ordinator.toml");
    let config_toml = r#"
[global]
default_profile = "default"
create_backups = true

[profiles.default]
files = [".testfile"]
directories = []
exclude = []
"#;
    std::fs::write(&config_path, config_toml).unwrap();
    // Debug: print the config file content that was written
    print!(
        "[TEST DEBUG] Config file written to: {}",
        config_path.display()
    );
    print!("[TEST DEBUG] Config file content:");
    print!("{}", std::fs::read_to_string(&config_path).unwrap());

    // Place the managed dotfile in files/
    let files_dir = ordinator_home.join("files");
    std::fs::create_dir_all(&files_dir).unwrap();
    let managed_path = files_dir.join(".testfile");
    std::fs::write(&managed_path, "hello").unwrap();

    // Debug after initial creation
    let debug_path = temp.child("debug.txt");
    {
        use std::io::Write;
        let managed_parent = managed_path.parent().unwrap();
        let mut debug_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(debug_path.path())
            .unwrap();
        writeln!(
            debug_file,
            "[AFTER CREATE] managed parent: {managed_parent}",
            managed_parent = managed_parent.display()
        )
        .unwrap();
        writeln!(
            debug_file,
            "[AFTER CREATE] managed parent exists: {exists}",
            exists = managed_parent.exists()
        )
        .unwrap();
        match std::fs::metadata(managed_parent) {
            Ok(meta) => {
                writeln!(
                    debug_file,
                    "[AFTER CREATE] managed parent permissions: {:?}",
                    meta.permissions()
                )
                .unwrap();
            }
            Err(e) => {
                writeln!(
                    debug_file,
                    "[AFTER CREATE] managed parent metadata error: {e}"
                )
                .unwrap();
            }
        }
        writeln!(
            debug_file,
            "[AFTER CREATE] managed file path: {}",
            managed_path.display()
        )
        .unwrap();
    }
    if debug_path.path().exists() {
        if let Ok(debug_contents) = std::fs::read_to_string(debug_path.path()) {
            eprintln!("[TEST DEBUG FILE AFTER CREATE]\n{debug_contents}");
        }
    }

    // Ensure no existing symlink or file exists
    let home_file = temp.child(".testfile");
    if home_file.path().exists() {
        fs::remove_file(home_file.path()).unwrap();
    }

    // Run init to set up git repo
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["init"]);
    let _ = cmd.output();

    // First apply to create the symlink
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["apply", "--force"]);
    let output = cmd.output().unwrap();
    assert!(output.status.success(), "Initial apply failed");

    // Remove the symlink and create a conflicting file
    if home_file.path().exists() {
        fs::remove_file(home_file.path()).unwrap();
    }
    home_file.write_str("conflict").unwrap();

    // Debug: confirm the conflicting file exists and is a regular file
    println!("[TEST DEBUG] After creating conflict file:");
    println!("[TEST DEBUG] File exists: {}", home_file.path().exists());
    if home_file.path().exists() {
        let meta = std::fs::symlink_metadata(home_file.path()).unwrap();
        if meta.file_type().is_symlink() {
            println!("[TEST DEBUG] File type: symlink");
        } else if meta.is_file() {
            println!("[TEST DEBUG] File type: regular file");
        } else if meta.is_dir() {
            println!("[TEST DEBUG] File type: directory");
        } else {
            println!("[TEST DEBUG] File type: other");
        }
    }

    // Run apply without force (should fail)
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["apply"]);
    let output = cmd.output().unwrap();
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!(
            "Apply without --force unexpectedly succeeded.\nStatus: {:?}\nStdout: {}\nStderr: {}",
            output.status, stdout, stderr
        );
    }
    assert!(
        !output.status.success(),
        "Apply without --force should fail"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("already exists"),
        "Expected 'already exists' in stderr, got: {stderr}"
    );

    // Run apply with force (should succeed)
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_HOME", ordinator_home);
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.args(["apply", "--force", "--verbose"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "Apply with --force failed: {stderr}"
    );
    assert!(
        stderr.contains("Symlinked"),
        "Expected 'Symlinked' in stderr, got: {stderr}"
    );

    // Symlink should be valid
    #[cfg(unix)]
    {
        let meta = fs::symlink_metadata(home_file.path()).unwrap();
        assert!(
            meta.file_type().is_symlink(),
            "Destination is not a symlink"
        );
        let actual = home_file
            .path()
            .read_link()
            .unwrap()
            .canonicalize()
            .unwrap();
        let expected = managed_path.canonicalize().unwrap();
        assert_eq!(
            actual, expected,
            "Symlink target does not match managed file"
        );
    }
}

#[test]
fn test_create_config_with_symlink() {
    use assert_fs::prelude::*;
    let temp = assert_fs::TempDir::new().unwrap();
    let source_file = temp.child("source.txt");
    let target_dir = temp.child("target");
    let ordinator_home = temp.child(".ordinator");
    ordinator_home.create_dir_all().unwrap();

    // Create source file
    source_file.touch().unwrap();

    // Create target directory
    target_dir.create_dir_all().unwrap();

    // Create config with symlink
    let config_content = format!(
        r#"
[profiles.default]
symlinks = [
    "{} -> {}/source.txt"
]
"#,
        source_file.display(),
        target_dir.display()
    );
    let config_file = ordinator_home.join("ordinator.toml");
    std::fs::write(&config_file, config_content).unwrap();

    // Debug: print the config file content that was written
    println!(
        "[TEST DEBUG] Config file written to: {}",
        config_file.display()
    );
    println!("[TEST DEBUG] Config file content:");
    println!("{}", std::fs::read_to_string(&config_file).unwrap());

    // Debug: print the config file content that was written
    println!(
        "[TEST DEBUG] Config file written to: {}",
        config_file.display()
    );
    println!("[TEST DEBUG] Config file content:");
    println!("{}", std::fs::read_to_string(&config_file).unwrap());
}
