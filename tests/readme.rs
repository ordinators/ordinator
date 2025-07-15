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

#[test]
fn test_readme_includes_homebrew_packages() {
    let temp = assert_fs::TempDir::new().unwrap();
    let custom_config = r#"
[global]
default_profile = "work"
auto_push = false
create_backups = true
exclude = []

[profiles.work]
files = ["~/.zshrc"]
homebrew_formulas = ["git", "neovim"]
homebrew_casks = ["iterm2"]
description = "Work profile"
directories = []
secrets = []
enabled = true
exclude = []

[profiles.personal]
files = ["~/.bashrc"]
homebrew_formulas = ["alacritty"]
homebrew_casks = []
description = "Personal profile"
directories = []
secrets = []
enabled = true
exclude = []

[secrets]
encrypt_patterns = []
exclude_patterns = []

[readme]
auto_update = false
update_on_changes = ["profiles", "bootstrap"]
"#;
    // Write the config directly
    let config_path = temp.child("ordinator.toml");
    std::fs::write(config_path.path(), custom_config).unwrap();
    // Set environment variables
    std::env::set_var("ORDINATOR_CONFIG", config_path.path());
    std::env::set_var("ORDINATOR_TEST_MODE", "1");
    std::env::set_var("ORDINATOR_HOME", temp.path());

    // Run the ordinator readme default command
    let mut cmd = assert_cmd::Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    cmd.env("ORDINATOR_CONFIG", config_path.path());
    cmd.env("ORDINATOR_TEST_MODE", "1");
    cmd.env("ORDINATOR_HOME", temp.path());
    cmd.args(["readme", "default"]);
    let output = cmd.output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "Readme default failed: {stderr}");

    // Check that README.md file was created and contains Homebrew section
    let readme_file = temp.child("README.md");
    assert!(
        readme_file.path().exists(),
        "README.md file was not created"
    );
    let readme_content = std::fs::read_to_string(readme_file.path()).unwrap();
    // Check for Homebrew Packages section and links
    if !readme_content.contains("## Homebrew Packages") {
        println!("\n[DEBUG] README CONTENT:\n{readme_content}\n");
    }
    assert!(
        readme_content.contains("## Homebrew Packages"),
        "README missing Homebrew Packages section"
    );
    assert!(
        readme_content.contains("<a href=\"https://formulae.brew.sh/formula/git\""),
        "README missing git formula link"
    );
    assert!(
        readme_content.contains("<a href=\"https://formulae.brew.sh/formula/neovim\""),
        "README missing neovim formula link"
    );
    assert!(
        readme_content.contains("<a href=\"https://formulae.brew.sh/cask/iterm2\""),
        "README missing iterm2 cask link"
    );
    assert!(
        readme_content.contains("<a href=\"https://formulae.brew.sh/formula/alacritty\""),
        "README missing alacritty formula link"
    );
}
