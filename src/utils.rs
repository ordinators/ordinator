use anyhow::Result;
use chrono::Local;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Utility functions for Ordinator
#[allow(dead_code)]
/// Get the home directory
pub fn get_home_dir() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("ORDINATOR_HOME") {
        return Ok(PathBuf::from(path));
    }
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))
}

#[allow(dead_code)]
/// Get the dotfiles directory
pub fn get_dotfiles_dir() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(".dotfiles"))
}

/// Create a symlink with backup
#[allow(dead_code)]
pub fn create_symlink_with_backup(source: &Path, target: &Path, backup: bool) -> Result<()> {
    if target.exists() {
        if backup {
            let backup_path = target.with_extension("backup");
            fs::rename(target, &backup_path)?;
        } else {
            fs::remove_file(target)?;
        }
    }

    // Create parent directories if they don't exist
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create symlink
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(source, target)?;

    Ok(())
}

/// Check if a path is a symlink
#[allow(dead_code)]
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata()
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(false)
}

/// Get the target of a symlink
#[allow(dead_code)]
pub fn get_symlink_target(path: &Path) -> Result<PathBuf> {
    Ok(fs::read_link(path)?)
}

/// Validate if a symlink is valid (exists and points to correct target)
#[allow(dead_code)]
pub fn validate_symlink(symlink_path: &Path, expected_target: &Path) -> Result<bool> {
    if !is_symlink(symlink_path) {
        return Ok(false);
    }

    let actual_target = get_symlink_target(symlink_path)?;
    Ok(actual_target == expected_target && expected_target.exists())
}

/// Check if a symlink is broken (target doesn't exist)
pub fn is_broken_symlink(path: &Path) -> bool {
    if !is_symlink(path) {
        return false;
    }

    match get_symlink_target(path) {
        Ok(target) => !target.exists(),
        Err(_) => true,
    }
}

/// Get the next backup number for a file
fn get_next_backup_number(backup_dir: &Path, filename: &str) -> Result<u32> {
    let mut number = 1;
    loop {
        let backup_name = format!(
            "{}.backup.{}.{}",
            filename,
            number,
            Local::now().format("%Y%m%d-%H%M%S")
        );
        let backup_path = backup_dir.join(backup_name);
        if !backup_path.exists() {
            return Ok(number);
        }
        number += 1;
        if number > 1000 {
            // Prevent infinite loops
            return Err(anyhow::anyhow!("Too many backup files for {}", filename));
        }
    }
}

/// Check if a file contains secrets (basic heuristic)
#[allow(dead_code)]
pub fn contains_secrets(content: &str) -> bool {
    let secret_patterns = [
        "password",
        "secret",
        "key",
        "token",
        "api_key",
        "private_key",
        "ssh_key",
    ];

    let lower_content = content.to_lowercase();
    secret_patterns
        .iter()
        .any(|pattern| lower_content.contains(pattern))
}

/// Back up a file to the dotfiles backup directory, appending a timestamp
#[allow(dead_code)]
pub fn backup_file_to_dotfiles_backup(original: &Path, config_path: &Path) -> Result<PathBuf> {
    let backup_dir = config_path.parent().unwrap().join("backups");
    std::fs::create_dir_all(&backup_dir)?;
    let filename = original.file_name().unwrap_or_default();
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("{}-{}", filename.to_string_lossy(), timestamp);
    let backup_path = backup_dir.join(backup_name);
    std::fs::copy(original, &backup_path)?;
    Ok(backup_path)
}

/// Enhanced backup with hybrid naming (number.timestamp)
pub fn backup_file_hybrid(original: &Path, config_path: &Path) -> Result<PathBuf> {
    let backup_dir = config_path.parent().unwrap().join("backups");
    std::fs::create_dir_all(&backup_dir)?;
    let filename = original.file_name().unwrap_or_default().to_string_lossy();
    let backup_number = get_next_backup_number(&backup_dir, &filename)?;
    let timestamp = Local::now().format("%Y%m%d-%H%M%S");
    let backup_name = format!("{filename}.backup.{backup_number}.{timestamp}");
    let backup_path = backup_dir.join(backup_name);
    std::fs::copy(original, &backup_path)?;
    Ok(backup_path)
}

/// Create symlink with enhanced conflict resolution
pub fn create_symlink_with_conflict_resolution(
    source: &Path,
    target: &Path,
    force: bool,
    backup: bool,
    config_path: &Path,
) -> Result<()> {
    if target.exists() {
        if is_symlink(target) {
            // Check if it's already the correct symlink
            if let Ok(actual_target) = get_symlink_target(target) {
                if actual_target == source {
                    return Ok(()); // Already correct
                }
            }
        } else {
            // If it's not a symlink, treat as conflict unless force is set
            if !force {
                eprintln!(
                    "[DEBUG] Conflict: target {} exists and is not a symlink. Returning error.",
                    target.display()
                );
                return Err(anyhow::anyhow!(
                    "Target {} already exists and is not a symlink. Use --force to overwrite.",
                    target.display()
                ));
            }
        }

        // Backup if enabled
        if backup {
            let backup_path = backup_file_hybrid(target, config_path)?;
            eprintln!(
                "Backed up {} to {}",
                target.display(),
                backup_path.display()
            );
        }

        // Remove the existing file/symlink
        if target.is_dir() {
            fs::remove_dir_all(target)?;
        } else {
            fs::remove_file(target)?;
        }
    }

    // Create parent directories if they don't exist
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create symlink
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, target)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(source, target)?;

    Ok(())
}

/// Repair a broken symlink
pub fn repair_symlink(symlink_path: &Path, expected_target: &Path) -> Result<()> {
    if !is_symlink(symlink_path) {
        return Err(anyhow::anyhow!(
            "Path is not a symlink: {}",
            symlink_path.display()
        ));
    }

    let actual_target = get_symlink_target(symlink_path)?;
    if actual_target == expected_target && expected_target.exists() {
        return Ok(()); // Already correct
    }

    // Remove the broken symlink
    fs::remove_file(symlink_path)?;

    // Create parent directories if they don't exist
    if let Some(parent) = symlink_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create the correct symlink
    #[cfg(unix)]
    std::os::unix::fs::symlink(expected_target, symlink_path)?;

    #[cfg(windows)]
    std::os::windows::fs::symlink_file(expected_target, symlink_path)?;

    Ok(())
}

/// Generate a 6-character SHA-256 hash from a file path
pub fn generate_file_hash(path: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path.as_bytes());
    let result = hasher.finalize();
    format!("{result:x}")[0..6].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::os::unix::fs as unix_fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_home_dir() {
        // Test with ORDINATOR_HOME set
        std::env::set_var("ORDINATOR_HOME", "/test/home");
        let home = get_home_dir().unwrap();
        assert_eq!(home, PathBuf::from("/test/home"));

        // Test without ORDINATOR_HOME (should use system home)
        std::env::remove_var("ORDINATOR_HOME");
        let home = get_home_dir().unwrap();
        assert!(home.exists());
    }

    #[test]
    fn test_get_dotfiles_dir() {
        std::env::set_var("ORDINATOR_HOME", "/test/home");
        let dotfiles = get_dotfiles_dir().unwrap();
        assert_eq!(dotfiles, PathBuf::from("/test/home/.dotfiles"));
        std::env::remove_var("ORDINATOR_HOME");
    }

    #[test]
    fn test_is_symlink() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        let symlink = dir.path().join("link.txt");

        // Create a regular file
        File::create(&file).unwrap();
        assert!(!is_symlink(&file));

        // Create a symlink
        unix_fs::symlink(&file, &symlink).unwrap();
        assert!(is_symlink(&symlink));

        // Test non-existent path
        assert!(!is_symlink(&dir.path().join("nonexistent")));
    }

    #[test]
    fn test_get_symlink_target() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let symlink = dir.path().join("link.txt");

        File::create(&target).unwrap();
        unix_fs::symlink(&target, &symlink).unwrap();

        let symlink_target = get_symlink_target(&symlink).unwrap();
        assert_eq!(symlink_target, target);

        // Test error case
        assert!(get_symlink_target(&target).is_err()); // Not a symlink
    }

    #[test]
    fn test_create_symlink_with_backup() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("source.txt");
        let target = dir.path().join("target.txt");

        // Create source file
        File::create(&source).unwrap();

        // Test creating symlink without backup
        create_symlink_with_backup(&source, &target, false).unwrap();
        assert!(is_symlink(&target));

        // Test creating symlink with backup (target already exists)
        let new_target = dir.path().join("new_target.txt");
        File::create(&new_target).unwrap();
        create_symlink_with_backup(&source, &new_target, true).unwrap();
        assert!(is_symlink(&new_target));

        // Check that backup was created
        let backup_files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().to_string_lossy().contains("backup"))
            .collect();
        assert!(!backup_files.is_empty());
    }

    #[test]
    fn test_contains_secrets() {
        // Test positive cases
        assert!(contains_secrets("password=secret123"));
        assert!(contains_secrets("API_KEY=abc123"));
        assert!(contains_secrets("private_key"));
        assert!(contains_secrets("ssh_key"));
        assert!(contains_secrets("token=xyz"));

        // Test negative cases
        assert!(!contains_secrets("hello world"));
        assert!(!contains_secrets(""));
        assert!(!contains_secrets("normal text"));

        // Test case insensitivity
        assert!(contains_secrets("PASSWORD=secret"));
        assert!(contains_secrets("Password=secret"));
    }

    #[test]
    fn test_backup_file_to_dotfiles_backup() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("ordinator.toml");
        let file = dir.path().join("file.txt");

        // Create config and file
        File::create(&config_path).unwrap();
        let mut f = File::create(&file).unwrap();
        writeln!(f, "test content").unwrap();

        let backup_path = backup_file_to_dotfiles_backup(&file, &config_path).unwrap();

        // Check backup was created
        assert!(backup_path.exists());
        assert!(backup_path.to_string_lossy().contains("backups"));
        assert!(backup_path.to_string_lossy().contains("file.txt-"));

        // Check content
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("test content"));
    }

    #[test]
    fn test_create_symlink_with_conflict_resolution() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("ordinator.toml");
        let source = dir.path().join("source.txt");
        let target = dir.path().join("target.txt");

        // Create config and source
        File::create(&config_path).unwrap();
        File::create(&source).unwrap();

        // Test creating new symlink
        create_symlink_with_conflict_resolution(&source, &target, false, false, &config_path)
            .unwrap();
        assert!(is_symlink(&target));

        // Test with existing correct symlink (should do nothing)
        create_symlink_with_conflict_resolution(&source, &target, false, false, &config_path)
            .unwrap();
        assert!(is_symlink(&target));

        // Test with existing file (should fail without force)
        let new_target = dir.path().join("new_target.txt");
        File::create(&new_target).unwrap();
        assert!(create_symlink_with_conflict_resolution(
            &source,
            &new_target,
            false,
            false,
            &config_path
        )
        .is_err());

        // Test with force
        create_symlink_with_conflict_resolution(&source, &new_target, true, false, &config_path)
            .unwrap();
        assert!(is_symlink(&new_target));
    }

    #[test]
    fn test_create_symlink_with_conflict_resolution_with_backup() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("ordinator.toml");
        let source = dir.path().join("source.txt");
        let target = dir.path().join("target.txt");

        // Create config and source
        File::create(&config_path).unwrap();
        File::create(&source).unwrap();

        // Create existing file
        File::create(&target).unwrap();

        // Test with backup enabled
        create_symlink_with_conflict_resolution(&source, &target, true, true, &config_path)
            .unwrap();
        assert!(is_symlink(&target));

        // Check backup was created
        let backup_dir = config_path.parent().unwrap().join("backups");
        assert!(backup_dir.exists());
        let backup_files: Vec<_> = fs::read_dir(&backup_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .collect();
        assert!(!backup_files.is_empty());
    }

    #[test]
    fn test_repair_symlink_errors() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("file.txt");
        let nonexistent = dir.path().join("nonexistent.txt");

        // Test repairing non-symlink
        File::create(&file).unwrap();
        assert!(repair_symlink(&file, &nonexistent).is_err());

        // Test repairing with non-existent target (broken symlink)
        let symlink = dir.path().join("link.txt");
        unix_fs::symlink(&nonexistent, &symlink).unwrap();
        // Should now succeed (removes the broken symlink)
        assert!(repair_symlink(&symlink, &nonexistent).is_ok());
        // The symlink should be removed
        assert!(!symlink.exists());
    }

    #[test]
    fn test_backup_file_hybrid_multiple_backups() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("ordinator.toml");
        let file = dir.path().join("file.txt");

        // Create config and file
        File::create(&config_path).unwrap();
        let mut f = File::create(&file).unwrap();
        writeln!(f, "test content").unwrap();

        // Create multiple backups
        let backup1 = backup_file_hybrid(&file, &config_path).unwrap();
        let backup2 = backup_file_hybrid(&file, &config_path).unwrap();
        let backup3 = backup_file_hybrid(&file, &config_path).unwrap();

        // All should exist and be different
        assert!(backup1.exists());
        assert!(backup2.exists());
        assert!(backup3.exists());
        assert_ne!(backup1, backup2);
        assert_ne!(backup2, backup3);
        assert_ne!(backup1, backup3);

        // Check naming pattern
        assert!(backup1.to_string_lossy().contains(".backup.1."));
        assert!(backup2.to_string_lossy().contains(".backup.2."));
        assert!(backup3.to_string_lossy().contains(".backup.3."));
    }

    #[test]
    fn test_validate_symlink_and_is_broken_symlink() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let symlink = dir.path().join("link.txt");
        File::create(&target).unwrap();
        unix_fs::symlink(&target, &symlink).unwrap();
        assert!(validate_symlink(&symlink, &target).unwrap());
        assert!(!is_broken_symlink(&symlink));
        // Remove target to break the symlink
        fs::remove_file(&target).unwrap();
        assert!(!validate_symlink(&symlink, &target).unwrap());
        assert!(is_broken_symlink(&symlink));
    }

    #[test]
    fn test_repair_symlink() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        let symlink = dir.path().join("link.txt");
        File::create(&target).unwrap();
        unix_fs::symlink(&target, &symlink).unwrap();
        // Remove target to break the symlink
        fs::remove_file(&target).unwrap();
        // Recreate target
        File::create(&target).unwrap();
        // Repair symlink
        repair_symlink(&symlink, &target).unwrap();
        assert!(validate_symlink(&symlink, &target).unwrap());
    }

    #[test]
    fn test_backup_file_hybrid() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("ordinator.toml");
        File::create(&config_path).unwrap();
        let file = dir.path().join("file.txt");
        let mut f = File::create(&file).unwrap();
        writeln!(f, "test").unwrap();
        let backup_path = backup_file_hybrid(&file, &config_path).unwrap();
        assert!(backup_path.exists());
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert!(backup_content.contains("test"));
    }

    #[test]
    fn test_create_symlink_with_nested_directories() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("ordinator.toml");
        let source = dir.path().join("source.txt");
        let target = dir.path().join("nested").join("deep").join("target.txt");

        // Create config and source
        File::create(&config_path).unwrap();
        File::create(&source).unwrap();

        // Test creating symlink in nested directory
        create_symlink_with_conflict_resolution(&source, &target, false, false, &config_path)
            .unwrap();
        assert!(is_symlink(&target));
        assert!(target.parent().unwrap().exists());
    }

    #[test]
    fn test_generate_file_hash_deterministic() {
        let path = "/Users/test/.zshrc";
        let hash1 = generate_file_hash(path);
        let hash2 = generate_file_hash(path);
        assert_eq!(
            hash1, hash2,
            "Hash should be deterministic for the same path"
        );
    }

    #[test]
    fn test_generate_file_hash_unique() {
        let path1 = "/Users/test/.zshrc";
        let path2 = "/Users/test/.bashrc";
        let hash1 = generate_file_hash(path1);
        let hash2 = generate_file_hash(path2);
        assert_ne!(
            hash1, hash2,
            "Different paths should produce different hashes"
        );
    }
}
