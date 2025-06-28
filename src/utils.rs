use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;

/// Utility functions for Ordinator

/// Get the home directory
pub fn get_home_dir() -> Result<PathBuf> {
    dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))
}

/// Get the dotfiles directory
pub fn get_dotfiles_dir() -> Result<PathBuf> {
    Ok(get_home_dir()?.join(".dotfiles"))
}

/// Create a symlink with backup
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
pub fn is_symlink(path: &Path) -> bool {
    path.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false)
}

/// Get the target of a symlink
pub fn get_symlink_target(path: &Path) -> Result<PathBuf> {
    Ok(fs::read_link(path)?)
}

/// Check if a file contains secrets (basic heuristic)
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
    secret_patterns.iter().any(|pattern| lower_content.contains(pattern))
} 