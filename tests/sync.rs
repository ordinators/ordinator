mod common;
use assert_cmd::Command;
use assert_fs::fixture::PathChild;

#[test]
fn test_sync_errors_without_config() {
    let temp = assert_fs::TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    let missing_config = temp.child("no-such-config.toml");
    cmd.env("ORDINATOR_CONFIG", missing_config.path());
    cmd.args(["sync"]);
    common::assert_config_error(cmd.assert().failure());
}

#[test]
fn test_sync_errors_without_git_repo() {
    let temp = assert_fs::TempDir::new().unwrap();
    let config_file = temp.child("ordinator.toml");
    let config_path = config_file.path().to_path_buf();
    std::fs::write(&config_path, "not a valid toml").unwrap();
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.current_dir(&temp);
    let _config_guard = common::EnvVarGuard::set("ORDINATOR_CONFIG", &config_path);
    cmd.args(["sync"]);
    common::assert_config_error(cmd.assert().failure());
}
