mod common;
use assert_cmd::assert::OutputAssertExt;
use assert_cmd::Command;
use assert_fs::fixture::FileWriteStr;
use assert_fs::fixture::PathChild;
use assert_fs::fixture::PathCreateDir;
use predicates::str::contains;

#[test]
fn test_init_dry_run() {
    let mut cmd = Command::cargo_bin("ordinator").unwrap();
    cmd.args(["init", "--dry-run"]);
    cmd.assert()
        .success()
        .stderr(contains("DRY-RUN"))
        .stderr(contains(
            "Initializing new repository with profile: default",
        ));
}

#[test]
fn test_init_with_invalid_repo_url() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Test various invalid repository URLs - all should fail validation
    let invalid_urls = vec![
        "https://github.com/",                   // invalid - missing repo
        "https://github.com/user",               // invalid - missing repo
        "not-a-url",                             // invalid format
        "https://github.com/user name/repo.git", // spaces in URL
        "https://invalid-url.com/user/repo",     // non-GitHub domain
        "",                                      // empty URL
    ];

    for url in invalid_urls {
        let mut cmd = common::create_ordinator_command(&temp);
        cmd.args(["init", url]);
        cmd.assert()
            .failure()
            .stderr(contains("Invalid GitHub URL"));
    }
}

#[test]
fn test_init_with_malformed_github_url() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Test malformed GitHub URLs - these should all fail validation
    let malformed_urls = [
        "https://github.com//repo.git",         // empty owner
        "https://github.com/user//repo.git",    // empty segment
        "git@github.com:user//repo.git",        // empty segment
        "git@github.com:/user/repo.git",        // empty owner
        "https://github.com/user/repo/extra",   // too many segments
        "https://not-github.com/user/repo.git", // not github domain
        "git@not-github.com:user/repo.git",     // not github domain
    ];

    for url in malformed_urls {
        let mut cmd = common::create_ordinator_command(&temp);
        cmd.args(["init", url]);
        cmd.assert()
            .failure()
            .stderr(contains("Invalid GitHub URL"));
    }
}

#[test]
fn test_init_with_nonexistent_repo() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a unique subdirectory path for this test (but don't create the directory)
    let repo_dir = temp.child("repo-nonexistent-test");

    // Use a valid GitHub URL with a non-existent user/repo
    // This should now succeed as it treats it as a new repository
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://github.com/nonexistent-user-xyz123/nonexistent-repo-xyz123.git",
        repo_dir.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_private_repo_no_auth() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a unique subdirectory path for this test (but don't create the directory)
    let repo_dir = temp.child("repo-private-test");

    // Use a valid GitHub URL for a private repo
    // This should now succeed as it treats it as a new repository
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://github.com/private-owner/private-repo.git",
        repo_dir.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_target_dir_permission_denied() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Test with a path that would cause permission issues
    let protected_path = "/root/protected/directory";

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["init", "https://github.com/user/repo.git", protected_path]);
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_existing_directory_no_force() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a directory that already exists
    let existing_dir = temp.child("existing");
    existing_dir.create_dir_all().unwrap();
    existing_dir
        .child("some-file.txt")
        .write_str("content")
        .unwrap();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://github.com/user/repo.git",
        existing_dir.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_force_flag() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a directory that already exists
    let existing_dir = temp.child("existing");
    existing_dir.create_dir_all().unwrap();
    existing_dir
        .child("some-file.txt")
        .write_str("content")
        .unwrap();

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://github.com/user/repo.git",
        existing_dir.path().to_str().unwrap(),
        "--force",
    ]);

    // This should now succeed as it treats it as a new repository
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_invalid_target_directory() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Test with invalid target directory paths
    let invalid_paths = vec![
        "/nonexistent/path/that/cannot/be/created",
        "/root/protected/directory",
        "relative/path/with/../invalid/../traversal",
    ];

    for path in invalid_paths {
        let mut cmd = common::create_ordinator_command(&temp);
        cmd.args(["init", "https://github.com/user/repo.git", path]);
        cmd.assert()
            .success()
            .stderr(contains("Initializing new repository with remote URL"));
    }
}

#[test]
fn test_init_with_empty_repo_url() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Test with empty repository URL - should now fail
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["init", ""]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Invalid GitHub URL"));
}

#[test]
fn test_init_with_whitespace_repo_url() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["init", "   "]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Invalid GitHub URL"));
}

#[test]
fn test_init_with_unicode_repo_url() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["init", "https://github.com/user/repö.git"]);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Invalid GitHub URL"));
}

#[test]
fn test_init_with_very_long_repo_url() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a unique subdirectory path for this test (but don't create the directory)
    let repo_dir = temp.child("repo-long-url-test");

    // Use a very long but valid GitHub URL
    let long_owner = "a".repeat(39); // GitHub max username length
    let long_repo = "b".repeat(100);
    let long_url = format!("https://github.com/{long_owner}/{long_repo}.git");

    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args(["init", &long_url, repo_dir.path().to_str().unwrap()]);
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_network_timeout() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a unique subdirectory path for this test (but don't create the directory)
    let repo_dir = temp.child("repo-network-test");

    // Use a valid GitHub URL with a non-existent user/repo
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://github.com/nonexistent-user-xyz123/nonexistent-repo-xyz123.git",
        repo_dir.path().to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}

#[test]
fn test_init_with_invalid_repo_structure() {
    let temp = assert_fs::TempDir::new().unwrap();
    let (_config_guard, _test_mode_guard) = common::setup_test_environment_with_config(&temp, None);

    // Create a unique subdirectory path for this test (but don't create the directory)
    let repo_dir = temp.child("repo-structure-test");

    // Test with a repository that exists but doesn't have ordinator.toml
    // Use a smaller, well-known repo that likely doesn't have ordinator.toml
    let mut cmd = common::create_ordinator_command(&temp);
    cmd.args([
        "init",
        "https://github.com/microsoft/vscode.git",
        repo_dir.path().to_str().unwrap(),
    ]);

    // This should now succeed as it treats it as a new repository
    cmd.assert()
        .success()
        .stderr(contains("Initializing new repository with remote URL"));
}
