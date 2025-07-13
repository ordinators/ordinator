mod common;
use assert_cmd::assert::OutputAssertExt;
use assert_fs::fixture::PathChild;
use assert_fs::fixture::PathCreateDir;
use predicates::str::contains;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_brew_export_and_list_with_dummy_brew() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create dummy brew script
    let brew_dir = temp.child("dummy_bin");
    brew_dir.create_dir_all().unwrap();
    let brew_path = brew_dir.child("brew");
    let mut brew_file = std::fs::File::create(brew_path.path()).unwrap();
    // Simulate 'brew list --formula' and 'brew list --cask'
    writeln!(brew_file, "#!/bin/sh").unwrap();
    writeln!(
        brew_file,
        "if [ \"$1\" = 'list' ] && [ \"$2\" = '--formula' ]; then echo 'dummyformula'; exit 0; fi"
    )
    .unwrap();
    writeln!(
        brew_file,
        "if [ \"$1\" = 'list' ] && [ \"$2\" = '--cask' ]; then echo 'dummycask'; exit 0; fi"
    )
    .unwrap();
    writeln!(
        brew_file,
        "if [ \"$1\" = '--version' ]; then echo 'Homebrew 3.0.0'; exit 0; fi"
    )
    .unwrap();
    writeln!(brew_file, "echo 'ok' >&2; exit 0").unwrap();
    let mut perms = std::fs::metadata(brew_path.path()).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(brew_path.path(), perms).unwrap();

    // Prepend dummy_bin to PATH for every command
    let old_path = std::env::var("PATH").unwrap();
    let new_path = format!("{}:{}", brew_dir.path().display(), old_path);
    let _path_guard = common::EnvVarGuard::set("PATH", &new_path);

    // Run ordinator brew export
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", &new_path);
    cmd.args(["brew", "export", "--profile", "default", "--force"]);
    cmd.assert()
        .success()
        .stderr(contains("Exported Homebrew packages to profile 'default'"));

    // Run ordinator brew list
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", &new_path);
    cmd.args(["brew", "list", "--profile", "default"]);
    cmd.assert()
        .success()
        .stdout(contains("dummyformula"))
        .stdout(contains("dummycask"));
}

#[test]
fn test_brew_install_and_apply_with_dummy_brew() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create dummy brew script
    let brew_dir = temp.child("dummy_bin");
    brew_dir.create_dir_all().unwrap();
    let brew_path = brew_dir.child("brew");
    let mut brew_file = std::fs::File::create(brew_path.path()).unwrap();
    // Simulate install and --version
    writeln!(brew_file, "#!/bin/sh").unwrap();
    writeln!(
        brew_file,
        "if [ \"$1\" = '--version' ]; then echo 'Homebrew 3.0.0'; exit 0; fi"
    )
    .unwrap();
    writeln!(
        brew_file,
        "if [ \"$1\" = 'install' ]; then echo 'installing $2'; exit 0; fi"
    )
    .unwrap();
    writeln!(brew_file, "if [ \"$1\" = 'install' ] && [ \"$2\" = '--cask' ]; then echo 'installing cask $3'; exit 0; fi").unwrap();
    writeln!(brew_file, "echo 'ok' >&2; exit 0").unwrap();
    let mut perms = std::fs::metadata(brew_path.path()).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(brew_path.path(), perms).unwrap();

    // Prepend dummy_bin to PATH for every command
    let old_path = std::env::var("PATH").unwrap();
    let new_path = format!("{}:{}", brew_dir.path().display(), old_path);
    let _path_guard = common::EnvVarGuard::set("PATH", &new_path);

    // First export some packages to create the config
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", &new_path);
    cmd.args(["brew", "export", "--profile", "default", "--force"]);
    cmd.assert()
        .success()
        .stderr(contains("Exported Homebrew packages to profile 'default'"));

    // Run ordinator brew install
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", &new_path);
    cmd.args(["brew", "install", "--profile", "default"]);
    cmd.assert().success().stderr(contains(
        "Homebrew package installation complete for profile 'default'",
    ));

    // Run ordinator apply --skip-brew (should NOT call brew install)
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", &new_path);
    cmd.args(["apply", "--profile", "default", "--skip-brew"]);
    cmd.assert()
        .success()
        .stderr(contains("Skipped Homebrew package installation"));

    // Run ordinator apply (should call dummy brew install)
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.env("PATH", &new_path);
    cmd.args(["apply", "--profile", "default"]);
    cmd.assert()
        .success()
        .stderr(contains("Homebrew packages installed successfully"));
}
