use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::*;
use predicates::str::contains;
use std::process::Command;

/// Environment variable guard that restores the original value when dropped
pub struct EnvVarGuard {
    key: String,
    original: Option<String>,
}

impl EnvVarGuard {
    #[allow(dead_code)]
    pub fn set<K: Into<String>, V: AsRef<std::ffi::OsStr>>(key: K, value: V) -> Self {
        let key = key.into();
        let original = std::env::var(&key).ok();
        std::env::set_var(&key, value);
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(ref original) = self.original {
            std::env::set_var(&self.key, original);
        } else {
            std::env::remove_var(&self.key);
        }
    }
}

/// Set up test environment with optional custom config
#[allow(dead_code)]
pub fn setup_test_environment_with_config(
    temp: &assert_fs::TempDir,
    custom_config: Option<&str>,
) -> (EnvVarGuard, EnvVarGuard) {
    setup_test_environment_with_config_and_test_name(temp, custom_config, None)
}

/// Set up test environment with optional custom config and test name
#[allow(dead_code)]
pub fn setup_test_environment_with_config_and_test_name(
    temp: &assert_fs::TempDir,
    custom_config: Option<&str>,
    test_name: Option<&str>,
) -> (EnvVarGuard, EnvVarGuard) {
    let config_guard = EnvVarGuard::set("ORDINATOR_CONFIG", temp.child("ordinator.toml").path());
    let test_mode_guard = EnvVarGuard::set("ORDINATOR_TEST_MODE", "1");

    // Initialize the repository
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(temp);
    cmd.env("ORDINATOR_CONFIG", temp.child("ordinator.toml").path());
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    if let Some(name) = test_name {
        cmd.env("ORDINATOR_TEST_NAME", name);
    }
    cmd.arg("init");
    let output = cmd.output().unwrap();
    println!("[DEBUG] ordinator init status: {:?}", output.status);
    println!(
        "[DEBUG] ordinator init stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    println!(
        "[DEBUG] ordinator init stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Write custom config if provided
    if let Some(config_content) = custom_config {
        std::fs::write(temp.child("ordinator.toml").path(), config_content).unwrap();
    }

    println!("[DEBUG] Temp dir: {:?}", temp.path());
    println!(
        "[DEBUG] Config file: {:?}",
        temp.child("ordinator.toml").path()
    );
    if let Ok(contents) = std::fs::read_to_string(temp.child("ordinator.toml").path()) {
        println!("[DEBUG] Config contents:\n{contents}");
    }

    (config_guard, test_mode_guard)
}

/// Create an ordinator command with proper test environment setup
#[allow(dead_code)]
pub fn create_ordinator_command(temp: &assert_fs::TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(temp);
    cmd.env("ORDINATOR_CONFIG", temp.child("ordinator.toml").path());
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd
}

/// Assert that the command failed with a config-related error
#[allow(dead_code)]
pub fn assert_config_error(assert: assert_cmd::assert::Assert) -> assert_cmd::assert::Assert {
    assert.stderr(
        contains("No configuration file found")
            .or(contains("Failed to parse config file"))
            .or(contains("No Git repository found")),
    )
}

/// Assert that the first TOML comment in the config matches the test name
#[allow(dead_code)]
pub fn assert_config_comment_matches(temp: &assert_fs::TempDir, test_name: &str) {
    let binding = temp.child("ordinator.toml");
    let config_path = binding.path();
    let contents = std::fs::read_to_string(config_path).expect("Failed to read ordinator.toml");
    let first_line = contents.lines().next().unwrap_or("");
    assert!(
        first_line.starts_with('#'),
        "First line should be a TOML comment"
    );
    let comment = first_line.trim_start_matches('#').trim();
    assert_eq!(
        comment,
        format!("test: {test_name}"),
        "TOML comment does not match test name"
    );
}
