use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use chrono::Local;

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
