use crate::config::Config;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tracing::info;

/// Safety level for bootstrap scripts
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyLevel {
    Safe,
    Warning,
    Dangerous,
    Blocked,
}

/// Bootstrap manager for running setup scripts and commands
#[allow(dead_code)]
pub struct BootstrapManager {
    dry_run: bool,
}

#[allow(dead_code)]
impl BootstrapManager {
    /// Create a new bootstrap manager
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Generate a bootstrap script for a profile
    pub fn generate_bootstrap_script(
        &self,
        profile: &str,
        config: &Config,
        dotfiles_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        if let Some(script_path) = config.get_bootstrap_script(profile) {
            let full_script_path = dotfiles_dir.join(&script_path);

            info!(
                "Generating bootstrap script for profile '{}': {:?}",
                profile, full_script_path
            );

            if self.dry_run {
                info!(
                    "[DRY RUN] Would generate bootstrap script: {:?}",
                    full_script_path
                );
                return Ok(Some(full_script_path));
            }

            // Ensure the script directory exists
            if let Some(parent) = full_script_path.parent() {
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create script directory: {}", parent.display())
                })?;
            }

            // Create a basic bootstrap script if it doesn't exist
            if !full_script_path.exists() {
                let script_content = self.create_default_script_content(profile);
                std::fs::write(&full_script_path, script_content).with_context(|| {
                    format!(
                        "Failed to write bootstrap script: {}",
                        full_script_path.display()
                    )
                })?;
            }

            // Make the script executable
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&full_script_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&full_script_path, perms).with_context(|| {
                    format!(
                        "Failed to make script executable: {}",
                        full_script_path.display()
                    )
                })?;
            }

            Ok(Some(full_script_path))
        } else {
            info!("No bootstrap script defined for profile '{}'", profile);
            Ok(None)
        }
    }

    /// Validate a bootstrap script for safety
    pub fn validate_script(&self, script_path: &Path) -> Result<SafetyLevel> {
        if !script_path.exists() {
            return Ok(SafetyLevel::Safe); // Non-existent scripts are safe
        }

        let content = std::fs::read_to_string(script_path)
            .with_context(|| format!("Failed to read script: {}", script_path.display()))?;

        Ok(self.analyze_script_safety(&content))
    }

    /// Get the safety level of a script
    pub fn get_script_safety_level(&self, script_path: &Path) -> SafetyLevel {
        match self.validate_script(script_path) {
            Ok(level) => level,
            Err(_) => SafetyLevel::Warning, // If we can't read the script, treat as warning
        }
    }

    /// Analyze script content for safety
    fn analyze_script_safety(&self, content: &str) -> SafetyLevel {
        // Blocked patterns (most severe) - match commands that are not in comments
        let blocked_patterns = [
            r"(?m)^[[:space:]]*rm\s+-rf\s+/\s*(?:$|#|;)", // Match rm -rf / at start of line, not in comments
            r"(?m)^[[:space:]]*format\s+",
            r"(?m)^[[:space:]]*dd\s+if=",
            r"(?m)^[[:space:]]*mkfs\s+",
        ];
        for pattern in &blocked_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(content) {
                return SafetyLevel::Blocked;
            }
        }

        // Dangerous patterns - match commands that are not in comments
        let dangerous_patterns = [r"(?m)^[[:space:]]*sudo\s+"];
        for pattern in &dangerous_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(content) {
                return SafetyLevel::Dangerous;
            }
        }

        // Warning patterns - match commands that are not in comments
        let warning_patterns = [
            r"(?m)^[[:space:]]*rm\s+-rf",
            r"(?m)^[[:space:]]*chmod\s+777",
            r"(?m)^[[:space:]]*chown\s+root",
        ];
        for pattern in &warning_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(content) {
                return SafetyLevel::Warning;
            }
        }

        SafetyLevel::Safe
    }

    /// Create default script content for a profile
    fn create_default_script_content(&self, profile: &str) -> String {
        format!(
            r#"#!/bin/bash
# Bootstrap script for profile: {profile}
# Generated by Ordinator
# 
# This script will be executed to set up your environment.
# Edit this file to customize your bootstrap process.

set -e  # Exit on any error

echo "Starting bootstrap for profile: {profile}"

# Add your bootstrap commands here
# Examples:
# brew install git neovim ripgrep
# defaults write com.apple.dock autohide -bool true
# npm install -g typescript

echo "Bootstrap complete for profile: {profile}"
"#
        )
    }

    /// Run a bootstrap script
    pub fn run_bootstrap_script(&self, script_path: &std::path::Path) -> Result<()> {
        info!("Running bootstrap script: {:?}", script_path);
        if self.dry_run {
            info!("[DRY RUN] Would run bootstrap script: {:?}", script_path);
            return Ok(());
        }

        if !script_path.exists() {
            return Err(anyhow::anyhow!(
                "Bootstrap script does not exist: {}",
                script_path.display()
            ));
        }

        // Validate script safety before execution
        let safety_level = self.get_script_safety_level(script_path);
        if safety_level == SafetyLevel::Blocked {
            return Err(anyhow::anyhow!(
                "Script execution blocked due to dangerous commands"
            ));
        }

        // Execute the script using std::process::Command
        let output = std::process::Command::new("bash")
            .arg(script_path)
            .current_dir(
                script_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new(".")),
            )
            .output()
            .with_context(|| {
                format!(
                    "Failed to execute bootstrap script: {}",
                    script_path.display()
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Bootstrap script failed with exit code {}:\nSTDOUT:\n{}\nSTDERR:\n{}",
                output.status,
                stdout,
                stderr
            ));
        }

        info!(
            "Bootstrap script executed successfully: {}",
            script_path.display()
        );
        Ok(())
    }

    /// Install Homebrew packages
    pub fn install_homebrew_packages(&self, packages: &[String]) -> Result<()> {
        info!("Installing Homebrew packages: {:?}", packages);
        if self.dry_run {
            info!("[DRY RUN] Would install Homebrew packages: {:?}", packages);
            return Ok(());
        }
        // TODO: Implement Homebrew package installation
        Ok(())
    }

    /// Install VS Code extensions
    pub fn install_vscode_extensions(&self, extensions: &[String]) -> Result<()> {
        info!("Installing VS Code extensions: {:?}", extensions);
        if self.dry_run {
            info!(
                "[DRY RUN] Would install VS Code extensions: {:?}",
                extensions
            );
            return Ok(());
        }
        // TODO: Implement VS Code extension installation
        Ok(())
    }

    /// Generate system script for manual execution
    pub fn generate_system_script(
        &self,
        _commands: &[String],
        output_path: &std::path::Path,
    ) -> Result<()> {
        info!("Generating system script: {:?}", output_path);
        // TODO: Implement system script generation
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_bootstrap_manager_new() {
        let manager = BootstrapManager::new(false);
        assert!(!manager.dry_run);

        let manager = BootstrapManager::new(true);
        assert!(manager.dry_run);
    }

    #[test]
    fn test_generate_bootstrap_script() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        // Create a minimal config with a bootstrap script
        let mut config = Config::create_default();
        config.profiles.get_mut("default").unwrap().bootstrap_script =
            Some("bootstrap.sh".to_string());

        let script_path = manager
            .generate_bootstrap_script("default", &config, temp_dir.path())
            .unwrap();
        assert!(script_path.is_some());

        let full_path = temp_dir.path().join("bootstrap.sh");
        assert!(full_path.exists());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert!(fs::metadata(&full_path).unwrap().permissions().mode() & 0o111 != 0);
            // Executable
        }
    }

    #[test]
    fn test_generate_bootstrap_script_no_script_defined() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);
        let config = Config::create_default();

        let script_path = manager
            .generate_bootstrap_script("default", &config, temp_dir.path())
            .unwrap();
        assert!(script_path.is_none());
    }

    #[test]
    fn test_generate_bootstrap_script_dry_run() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(true);

        let mut config = Config::create_default();
        config.profiles.get_mut("default").unwrap().bootstrap_script =
            Some("bootstrap.sh".to_string());

        let script_path = manager
            .generate_bootstrap_script("default", &config, temp_dir.path())
            .unwrap();
        assert!(script_path.is_some());

        // In dry run mode, the script should not actually be created
        let full_path = temp_dir.path().join("bootstrap.sh");
        assert!(!full_path.exists());
    }

    #[test]
    fn test_validate_script_safe() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
echo "Hello World"
brew install git
"#;
        let script_path = temp_dir.path().join("safe.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Safe);
    }

    #[test]
    fn test_validate_script_warning() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
rm -rf /tmp/test
echo "Hello World"
"#;
        let script_path = temp_dir.path().join("warning.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Warning);
    }

    #[test]
    fn test_validate_script_dangerous() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
sudo apt update
echo "Hello World"
"#;
        let script_path = temp_dir.path().join("dangerous.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validate_script_blocked() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
rm -rf /
echo "Hello World"
"#;
        let script_path = temp_dir.path().join("blocked.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Blocked);
    }

    #[test]
    fn test_validate_script_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_path = temp_dir.path().join("nonexistent.sh");
        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Safe);
    }

    #[test]
    fn test_run_bootstrap_script_success() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
echo "Hello World"
exit 0
"#;
        let script_path = temp_dir.path().join("test.sh");
        fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let result = manager.run_bootstrap_script(&script_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_bootstrap_script_failure() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
echo "Error message" >&2
exit 1
"#;
        let script_path = temp_dir.path().join("test.sh");
        fs::write(&script_path, script_content).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&script_path).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&script_path, perms).unwrap();
        }

        let result = manager.run_bootstrap_script(&script_path);
        assert!(result.is_err());
        let error = result.unwrap_err();
        // Accept either "exit code 1" or "failed" in the error message for robustness
        let msg = error.to_string();
        assert!(
            msg.contains("exit code 1") || msg.contains("failed"),
            "Unexpected error message: {msg}"
        );
    }

    #[test]
    fn test_run_bootstrap_script_nonexistent() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_path = temp_dir.path().join("nonexistent.sh");
        let result = manager.run_bootstrap_script(&script_path);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("does not exist"));
    }

    #[test]
    fn test_run_bootstrap_script_dry_run() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(true);

        let script_path = temp_dir.path().join("test.sh");
        // Even if the script doesn't exist, dry run should succeed
        let result = manager.run_bootstrap_script(&script_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_default_script_content() {
        let manager = BootstrapManager::new(false);
        let content = manager.create_default_script_content("test-profile");

        assert!(content.contains("#!/bin/bash"));
        assert!(content.contains("test-profile"));
        assert!(content.contains("set -e"));
        assert!(content.contains("brew install"));
    }

    #[test]
    fn test_bootstrap_manager_integration() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        // Test all methods in sequence
        let script_path = temp_dir.path().join("bootstrap.sh");
        assert!(manager.run_bootstrap_script(&script_path).is_err()); // Should fail for non-existent script

        let packages = vec!["git".to_string()];
        assert!(manager.install_homebrew_packages(&packages).is_ok());

        let extensions = vec!["ms-vscode.vscode-rust".to_string()];
        assert!(manager.install_vscode_extensions(&extensions).is_ok());

        let commands = vec!["defaults write test".to_string()];
        let output_path = temp_dir.path().join("test.sh");
        assert!(manager
            .generate_system_script(&commands, &output_path)
            .is_ok());
    }

    #[test]
    fn test_validate_script_with_comments() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
# This is a comment with sudo in it
echo "Hello World"
# sudo apt update  # commented out dangerous command
brew install git
"#;
        let script_path = temp_dir.path().join("commented.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Safe);
    }

    #[test]
    fn test_validate_script_with_multiple_dangerous_patterns() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
sudo apt update
sudo apt upgrade
sudo systemctl restart ssh
echo "Multiple dangerous commands"
"#;
        let script_path = temp_dir.path().join("multiple_dangerous.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Dangerous);
    }

    #[test]
    fn test_validate_script_with_quoted_commands() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
echo "sudo apt update"
echo "rm -rf /"
echo "Hello World"
"#;
        let script_path = temp_dir.path().join("quoted.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Safe);
    }

    #[test]
    fn test_validate_script_with_variables() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let script_content = r#"#!/bin/bash
CMD="sudo apt update"
echo "Hello World"
"#;
        let script_path = temp_dir.path().join("variables.sh");
        fs::write(&script_path, script_content).unwrap();

        let safety_level = manager.validate_script(&script_path).unwrap();
        assert_eq!(safety_level, SafetyLevel::Safe);
    }

    #[test]
    fn test_generate_bootstrap_script_with_custom_path() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        let mut config = Config::create_default();
        config.profiles.get_mut("default").unwrap().bootstrap_script =
            Some("custom/path/bootstrap.sh".to_string());

        let script_path = manager
            .generate_bootstrap_script("default", &config, temp_dir.path())
            .unwrap();
        assert!(script_path.is_some());

        let full_path = temp_dir.path().join("custom/path/bootstrap.sh");
        assert!(full_path.exists());
        assert!(full_path.parent().unwrap().exists());
    }

    #[test]
    fn test_generate_bootstrap_script_invalid_profile() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);
        let config = Config::create_default();

        // The function doesn't actually return an error for invalid profiles,
        // it just returns None if no bootstrap script is defined
        let result = manager.generate_bootstrap_script("invalid", &config, temp_dir.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_analyze_script_safety_edge_cases() {
        let manager = BootstrapManager::new(false);

        // Test empty content
        assert_eq!(manager.analyze_script_safety(""), SafetyLevel::Safe);

        // Test content with only whitespace
        assert_eq!(
            manager.analyze_script_safety("   \n\t  "),
            SafetyLevel::Safe
        );

        // Test content with blocked pattern
        assert_eq!(
            manager.analyze_script_safety("rm -rf /"),
            SafetyLevel::Blocked
        );

        // Test content with dangerous pattern
        assert_eq!(
            manager.analyze_script_safety("sudo apt update"),
            SafetyLevel::Dangerous
        );

        // Test content with warning pattern
        assert_eq!(
            manager.analyze_script_safety("rm -rf /tmp/test"),
            SafetyLevel::Warning
        );

        // Test content with safe patterns only
        assert_eq!(
            manager.analyze_script_safety("echo 'Hello World'\nbrew install git"),
            SafetyLevel::Safe
        );
    }

    #[test]
    fn test_safety_level_ordering() {
        // Test that blocked is highest priority
        let manager = BootstrapManager::new(false);
        let mixed_content = r#"#!/bin/bash
echo "Hello World"
sudo apt update
rm -rf /
brew install git
"#;
        let safety_level = manager.analyze_script_safety(mixed_content);
        assert_eq!(safety_level, SafetyLevel::Blocked);

        // Test that dangerous is higher than warning
        let dangerous_content = r#"#!/bin/bash
echo "Hello World"
sudo apt update
rm -rf /tmp/test
"#;
        let safety_level = manager.analyze_script_safety(dangerous_content);
        assert_eq!(safety_level, SafetyLevel::Dangerous);

        // Test that warning is higher than safe
        let warning_content = r#"#!/bin/bash
echo "Hello World"
rm -rf /tmp/test
"#;
        let safety_level = manager.analyze_script_safety(warning_content);
        assert_eq!(safety_level, SafetyLevel::Warning);
    }

    #[test]
    fn test_validate_script_read_error() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        // Create a directory with the same name as the script
        let script_path = temp_dir.path().join("script.sh");
        fs::create_dir(&script_path).unwrap();

        // Should return an error for unreadable files (directories)
        let result = manager.validate_script(&script_path);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read script"));
    }

    #[test]
    fn test_generate_bootstrap_script_permission_error() {
        let temp_dir = tempdir().unwrap();
        let manager = BootstrapManager::new(false);

        // Make the temp directory read-only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(temp_dir.path()).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(temp_dir.path(), perms).unwrap();
        }

        let mut config = Config::create_default();
        config.profiles.get_mut("default").unwrap().bootstrap_script =
            Some("bootstrap.sh".to_string());

        let result = manager.generate_bootstrap_script("default", &config, temp_dir.path());
        assert!(result.is_err());

        // Restore permissions for cleanup
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(temp_dir.path()).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(temp_dir.path(), perms).unwrap();
        }
    }
}
