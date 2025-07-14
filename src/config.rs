use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::readme::ReadmeConfig;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    /// Global configuration
    #[serde(default)]
    pub global: GlobalConfig,

    /// Profile-specific configurations
    #[serde(default)]
    pub profiles: HashMap<String, ProfileConfig>,

    /// Secrets configuration
    #[serde(default)]
    pub secrets: SecretsConfig,

    /// README configuration
    #[serde(default)]
    pub readme: ReadmeConfig,

    /// Unique identifier for this configuration (used for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    /// Default profile to use
    #[serde(default = "default_profile")]
    pub default_profile: String,

    /// Whether to auto-push after successful operations
    #[serde(default)]
    pub auto_push: bool,

    /// Whether to create backups before making changes
    #[serde(default)]
    pub create_backups: Option<bool>,

    /// Patterns for files/directories to exclude globally
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_profile: default_profile(),
            auto_push: false,
            create_backups: Some(default_backup()),
            exclude: Vec::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ProfileConfig {
    /// Files to track for this profile
    #[serde(default)]
    pub files: Vec<String>,

    /// Directories to track for this profile
    #[serde(default)]
    pub directories: Vec<String>,

    /// Secret files to track for this profile (direct paths to source files)
    #[serde(default)]
    pub secrets: Vec<String>,

    /// Bootstrap script for this profile
    pub bootstrap_script: Option<String>,

    /// Whether this profile is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Profile description
    pub description: Option<String>,

    /// Patterns for files/directories to exclude in this profile
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Homebrew packages for this profile
    #[serde(default)]
    pub homebrew_packages: Vec<String>,

    /// Date/time when the age key was created (ISO 8601 string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SecretsConfig {
    /// Age key file path
    pub age_key_file: Option<PathBuf>,

    /// SOPS configuration file path
    pub sops_config: Option<PathBuf>,

    /// Patterns for files that should be encrypted
    #[serde(default)]
    pub encrypt_patterns: Vec<String>,

    /// Patterns for files that should not be encrypted
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Key rotation interval in days (e.g., 90 = 3 months)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_rotation_interval_days: Option<u32>,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // Print identifier if present (for debugging)
        if let Some(identifier) = &config.identifier {
            eprintln!("[DEBUG] Loaded config: {identifier}");
        }

        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let mut content =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config to TOML")?;

        // Add identifier as a comment if present
        if let Some(identifier) = &self.identifier {
            content = format!("# {identifier}\n{content}");
        }

        // Ensure the directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        std::fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Find the configuration file in the current directory or dotfiles directory
    pub fn find_config_file() -> Result<Option<PathBuf>> {
        if let Ok(path) = env::var("ORDINATOR_CONFIG") {
            let pb = PathBuf::from(&path);
            if pb.exists() {
                return Ok(Some(pb));
            } else {
                return Ok(None);
            }
        }
        // Look for ordinator.toml in current directory
        let current_config = std::env::current_dir()?.join("ordinator.toml");
        if current_config.exists() {
            return Ok(Some(current_config));
        }

        // Look for ordinator.toml in dotfiles directory
        if let Ok(dotfiles_dir) = get_dotfiles_dir() {
            let dotfiles_config = dotfiles_dir.join("ordinator.toml");
            if dotfiles_config.exists() {
                return Ok(Some(dotfiles_config));
            }
        }

        Ok(None)
    }

    /// Load configuration from the standard location
    pub fn load() -> Result<(Self, PathBuf)> {
        if let Some(config_path) = Self::find_config_file()? {
            Ok((Self::from_file(&config_path)?, config_path))
        } else {
            Err(anyhow::anyhow!(
                "No configuration file found. Run 'ordinator init' first."
            ))
        }
    }

    /// Load configuration from file or return default if not found
    pub fn from_file_or_default() -> Result<Self> {
        if let Some(config_path) = Self::find_config_file()? {
            Self::from_file(&config_path)
        } else {
            Ok(Self::create_default())
        }
    }

    /// Create a default configuration
    pub fn create_default() -> Self {
        Self::create_default_with_identifier(None)
    }

    /// Create a default configuration with a test identifier
    pub fn create_default_with_identifier(test_name: Option<&str>) -> Self {
        let mut profiles = HashMap::new();

        // Add default profile
        profiles.insert(
            "default".to_string(),
            ProfileConfig {
                files: vec![],
                directories: vec![],
                bootstrap_script: None,
                enabled: true,
                description: Some("Default profile for basic dotfiles".to_string()),
                exclude: Vec::new(),
                homebrew_packages: Vec::new(),
                secrets: Vec::new(),
                created_on: None,
            },
        );

        // Add work profile
        profiles.insert(
            "work".to_string(),
            ProfileConfig {
                files: vec![],
                directories: vec![],
                bootstrap_script: None,
                enabled: true,
                description: Some("Work environment profile".to_string()),
                exclude: Vec::new(),
                homebrew_packages: Vec::new(),
                secrets: Vec::new(),
                created_on: None,
            },
        );

        // Add personal profile
        profiles.insert(
            "personal".to_string(),
            ProfileConfig {
                files: vec![],
                directories: vec![],
                bootstrap_script: None,
                enabled: true,
                description: Some("Personal environment profile".to_string()),
                exclude: Vec::new(),
                homebrew_packages: Vec::new(),
                secrets: Vec::new(),
                created_on: None,
            },
        );

        Config {
            global: GlobalConfig::default(),
            profiles,
            secrets: SecretsConfig::default(),
            readme: ReadmeConfig::default(),
            identifier: test_name.map(|name| format!("test: {name}")),
        }
    }

    /// Initialize a new configuration in the dotfiles directory
    #[allow(dead_code)]
    pub fn init_dotfiles_repository() -> Result<PathBuf> {
        Self::init_dotfiles_repository_with_test_name(None)
    }

    /// Initialize a new configuration in the dotfiles directory with test identifier
    pub fn init_dotfiles_repository_with_test_name(test_name: Option<&str>) -> Result<PathBuf> {
        if let Ok(path) = env::var("ORDINATOR_CONFIG") {
            let config_path = PathBuf::from(&path);
            let repo_dir = config_path.parent().unwrap();
            // Create parent directory if it doesn't exist
            std::fs::create_dir_all(repo_dir).with_context(|| {
                format!(
                    "Failed to create dotfiles directory: {}",
                    repo_dir.display()
                )
            })?;
            // Create config file
            let config = Self::create_default_with_identifier(test_name);
            config.save_to_file(&config_path)?;
            // Create subdirectories
            let scripts_dir = repo_dir.join("scripts");
            std::fs::create_dir_all(&scripts_dir).with_context(|| {
                format!(
                    "Failed to create scripts directory: {}",
                    scripts_dir.display()
                )
            })?;
            let files_dir = repo_dir.join("files");
            std::fs::create_dir_all(&files_dir).with_context(|| {
                format!("Failed to create files directory: {}", files_dir.display())
            })?;

            // Create .gitignore file
            Self::create_gitignore(repo_dir)?;

            return Ok(config_path);
        }
        let dotfiles_dir = get_dotfiles_dir()?;

        // Create dotfiles directory if it doesn't exist
        std::fs::create_dir_all(&dotfiles_dir).with_context(|| {
            format!(
                "Failed to create dotfiles directory: {}",
                dotfiles_dir.display()
            )
        })?;

        // Create config file
        let config_path = dotfiles_dir.join("ordinator.toml");
        let config = Self::create_default_with_identifier(test_name);
        config.save_to_file(&config_path)?;

        // Create subdirectories
        let scripts_dir = dotfiles_dir.join("scripts");
        std::fs::create_dir_all(&scripts_dir).with_context(|| {
            format!(
                "Failed to create scripts directory: {}",
                scripts_dir.display()
            )
        })?;

        let files_dir = dotfiles_dir.join("files");
        std::fs::create_dir_all(&files_dir).with_context(|| {
            format!("Failed to create files directory: {}", files_dir.display())
        })?;

        // Create .gitignore file
        Self::create_gitignore(&dotfiles_dir)?;

        Ok(config_path)
    }

    /// Get a profile configuration
    pub fn get_profile(&self, profile_name: &str) -> Option<&ProfileConfig> {
        self.profiles.get(profile_name)
    }

    /// Get a mutable profile configuration
    pub fn get_profile_mut(&mut self, profile_name: &str) -> Option<&mut ProfileConfig> {
        self.profiles.get_mut(profile_name)
    }

    /// Add a new profile
    #[allow(dead_code)]
    pub fn add_profile(&mut self, name: String, config: ProfileConfig) {
        self.profiles.insert(name, config);
    }

    /// Remove a profile
    #[allow(dead_code)]
    pub fn remove_profile(&mut self, name: &str) -> Option<ProfileConfig> {
        self.profiles.remove(name)
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<&String> {
        self.profiles.keys().collect()
    }

    /// Check if a profile exists
    #[allow(dead_code)]
    pub fn has_profile(&self, name: &str) -> bool {
        self.profiles.contains_key(name)
    }

    /// Get the default profile name
    #[allow(dead_code)]
    pub fn default_profile(&self) -> &str {
        &self.global.default_profile
    }

    /// Set the default profile
    #[allow(dead_code)]
    pub fn set_default_profile(&mut self, profile: String) {
        self.global.default_profile = profile;
    }

    /// Add a file to a profile's tracked files
    pub fn add_file_to_profile(&mut self, profile_name: &str, file_path: String) -> Result<()> {
        let profile = self
            .profiles
            .get_mut(profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' does not exist", profile_name))?;

        if !profile.files.contains(&file_path) {
            profile.files.push(file_path);
        }

        Ok(())
    }

    /// Add a secret file to a profile's tracked secrets
    pub fn add_secret_to_profile(&mut self, profile_name: &str, secret_path: String) -> Result<()> {
        let profile = self
            .profiles
            .get_mut(profile_name)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' does not exist", profile_name))?;

        if !profile.secrets.contains(&secret_path) {
            profile.secrets.push(secret_path);
        }

        Ok(())
    }

    /// Get the profile-specific file path for a tracked file
    pub fn get_profile_file_path(&self, profile_name: &str, file_path: &str) -> Result<PathBuf> {
        if !self.has_profile(profile_name) {
            return Err(anyhow::anyhow!("Profile '{}' not found", profile_name));
        }

        // For profile-specific storage, files are stored as files/<profile>/<file>
        let dotfiles_dir = get_dotfiles_dir()?;
        let profile_files_dir = dotfiles_dir.join("files").join(profile_name);
        let profile_file_path = profile_files_dir.join(file_path);

        Ok(profile_file_path)
    }

    /// Get the source file path for symlinking (handles both flat and profile-specific structures)
    pub fn get_source_file_path(&self, profile_name: &str, file_path: &str) -> Result<PathBuf> {
        let dotfiles_dir = get_dotfiles_dir()?;

        // First check if profile-specific file exists
        let profile_file_path = dotfiles_dir
            .join("files")
            .join(profile_name)
            .join(file_path);
        if profile_file_path.exists() {
            return Ok(profile_file_path);
        }

        // Fall back to flat structure for backward compatibility
        let flat_file_path = dotfiles_dir.join("files").join(file_path);
        if flat_file_path.exists() {
            return Ok(flat_file_path);
        }

        // If neither exists, return the profile-specific path (for new files)
        Ok(profile_file_path)
    }

    /// Remove a file from a profile
    pub fn remove_file_from_profile(&mut self, profile_name: &str, file_path: &str) -> Result<()> {
        if let Some(profile) = self.get_profile_mut(profile_name) {
            profile.files.retain(|f| f != file_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Profile '{}' not found", profile_name))
        }
    }

    /// Remove a secret file from a profile
    pub fn remove_secret_from_profile(
        &mut self,
        profile_name: &str,
        secret_path: &str,
    ) -> Result<()> {
        if let Some(profile) = self.get_profile_mut(profile_name) {
            profile.secrets.retain(|s| s != secret_path);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Profile '{}' not found", profile_name))
        }
    }

    /// Get the effective exclusion GlobSet for a profile (merges global and profile excludes)
    pub fn exclusion_set_for_profile(&self, profile_name: &str) -> anyhow::Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for pat in &self.global.exclude {
            builder.add(Glob::new(pat)?);
        }
        if let Some(profile) = self.get_profile(profile_name) {
            for pat in &profile.exclude {
                builder.add(Glob::new(pat)?);
            }
        }
        Ok(builder.build()?)
    }

    /// Get the bootstrap script path for a profile
    pub fn get_bootstrap_script(&self, profile: &str) -> Option<PathBuf> {
        self.get_profile(profile)
            .and_then(|profile_config| profile_config.bootstrap_script.as_ref())
            .map(|script_path| {
                // If the script path is relative, it should be relative to the dotfiles directory
                if script_path.starts_with('/') {
                    PathBuf::from(script_path)
                } else {
                    // For relative paths, we'll need the dotfiles directory
                    // This will be resolved when we have the config file path
                    PathBuf::from(script_path)
                }
            })
    }

    /// Create a .gitignore file for the dotfiles repository
    fn create_gitignore(dotfiles_dir: &Path) -> Result<()> {
        let gitignore_path = dotfiles_dir.join(".gitignore");

        // Don't overwrite existing .gitignore
        if gitignore_path.exists() {
            return Ok(());
        }

        let gitignore_content = r#"# Ordinator dotfiles repository .gitignore

# Sensitive files and keys
*.key
*.pem
*.p12
*.pfx
*.crt
*.cert
*.p8
*.keystore
*.jks

# Age encryption keys
age.key
age.txt
*.age

# SOPS encrypted files
*.enc.yaml
*.enc.yml
*.enc.json
*.enc.toml
*.enc.txt

# Backup files
*.bak
*.backup
*.old
*.orig
*.tmp
*.temp

# OS generated files
.DS_Store
.DS_Store?
._*
.Spotlight-V100
.Trashes
ehthumbs.db
Thumbs.db

# Editor files
*.swp
*.swo
*~
.vscode/
.idea/
*.sublime-*

# Log files
*.log
logs/

# Temporary files
*.tmp
*.temp
temp/
tmp/
"#;

        std::fs::write(&gitignore_path, gitignore_content).with_context(|| {
            format!(
                "Failed to write .gitignore file: {}",
                gitignore_path.display()
            )
        })?;

        println!("✅ Created .gitignore file: {}", gitignore_path.display());
        Ok(())
    }
}

// Helper functions for default values
fn default_profile() -> String {
    "default".to_string()
}

fn default_backup() -> bool {
    true
}

fn default_enabled() -> bool {
    true
}

/// Get the dotfiles directory path
fn get_dotfiles_dir() -> Result<PathBuf> {
    // Check if we're in test mode
    let is_test_mode = std::env::var("ORDINATOR_TEST_MODE").unwrap_or_default() == "1";

    if let Ok(ordinator_home) = std::env::var("ORDINATOR_HOME") {
        return Ok(PathBuf::from(ordinator_home));
    }

    // In test mode, require explicit configuration to prevent accidental use of real directories
    if is_test_mode {
        return Err(anyhow::anyhow!(
            "Test mode requires ORDINATOR_HOME or ORDINATOR_CONFIG to be set for proper isolation"
        ));
    }

    let home_dir =
        dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    Ok(home_dir.join(".dotfiles"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_creation() {
        let config = Config::create_default();
        assert!(config.has_profile("default"));
        assert!(config.has_profile("work"));
        assert!(config.has_profile("personal"));
        assert_eq!(config.default_profile(), "default");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::create_default();
        let temp_file = NamedTempFile::new().unwrap();

        // Save config
        config.save_to_file(temp_file.path()).unwrap();

        // Load config
        let loaded_config = Config::from_file(temp_file.path()).unwrap();

        assert_eq!(config.default_profile(), loaded_config.default_profile());
        assert_eq!(config.profiles.len(), loaded_config.profiles.len());
    }

    #[test]
    fn test_profile_management() {
        let mut config = Config::create_default();

        // Add a new profile
        let new_profile = ProfileConfig {
            files: vec!["test.txt".to_string()],
            directories: vec![],
            bootstrap_script: None,
            enabled: true,
            description: Some("Test profile".to_string()),
            exclude: Vec::new(),
            homebrew_packages: Vec::new(),
            secrets: Vec::new(),
            created_on: None,
        };

        config.add_profile("test".to_string(), new_profile);
        assert!(config.has_profile("test"));

        // Get profile
        let profile = config.get_profile("test").unwrap();
        assert_eq!(profile.files.len(), 1);
        assert_eq!(profile.files[0], "test.txt");

        // Remove profile
        let removed = config.remove_profile("test");
        assert!(removed.is_some());
        assert!(!config.has_profile("test"));
    }

    #[test]
    fn test_file_management() {
        let mut config = Config::create_default();

        // Add file to default profile
        config
            .add_file_to_profile("default", "~/.zshrc".to_string())
            .unwrap();
        let profile = config.get_profile("default").unwrap();
        assert!(profile.files.contains(&"~/.zshrc".to_string()));

        // Remove file from default profile
        config
            .remove_file_from_profile("default", "~/.zshrc")
            .unwrap();
        let profile = config.get_profile("default").unwrap();
        assert!(!profile.files.contains(&"~/.zshrc".to_string()));
    }

    #[test]
    fn test_secret_management() {
        let mut config = Config::create_default();

        // Add secret to default profile
        config
            .add_secret_to_profile("default", "~/.ssh/id_rsa".to_string())
            .unwrap();
        let profile = config.get_profile("default").unwrap();
        assert!(profile.secrets.contains(&"~/.ssh/id_rsa".to_string()));

        // Add another secret to the same profile
        config
            .add_secret_to_profile("default", "~/.ssh/id_rsa".to_string())
            .unwrap();
        let profile = config.get_profile("default").unwrap();
        assert!(profile.secrets.contains(&"~/.ssh/id_rsa".to_string()));
        assert_eq!(profile.secrets.len(), 1); // Should be unique

        // Remove secret from default profile
        config
            .remove_secret_from_profile("default", "~/.ssh/id_rsa")
            .unwrap();
        let profile = config.get_profile("default").unwrap();
        assert!(!profile.secrets.contains(&"~/.ssh/id_rsa".to_string()));
    }

    #[test]
    fn test_error_loading_malformed_config() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write invalid TOML
        writeln!(temp_file, "not a valid toml").unwrap();
        let result = Config::from_file(temp_file.path());
        assert!(result.is_err(), "Malformed config should return an error");
    }

    #[test]
    fn test_add_file_to_nonexistent_profile() {
        let mut config = Config::create_default();
        let result = config.add_file_to_profile("does_not_exist", "file.txt".to_string());
        assert!(
            result.is_err(),
            "Adding file to non-existent profile should error"
        );
    }

    #[test]
    fn test_remove_file_from_nonexistent_profile() {
        let mut config = Config::create_default();
        let result = config.remove_file_from_profile("does_not_exist", "file.txt");
        assert!(
            result.is_err(),
            "Removing file from non-existent profile should error"
        );
    }

    #[test]
    fn test_remove_nonexistent_file_from_profile() {
        let mut config = Config::create_default();
        // Removing a file that doesn't exist should not error
        let result = config.remove_file_from_profile("default", "not_in_profile.txt");
        assert!(
            result.is_ok(),
            "Removing non-existent file should be a no-op"
        );
        let profile = config.get_profile("default").unwrap();
        assert!(!profile.files.contains(&"not_in_profile.txt".to_string()));
    }
}
