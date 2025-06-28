use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Default profile to use
    #[serde(default = "default_profile")]
    pub default_profile: String,

    /// Whether to auto-push after successful apply
    #[serde(default)]
    pub auto_push: bool,

    /// Whether to backup existing files before symlinking
    #[serde(default = "default_backup")]
    pub backup_existing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileConfig {
    /// Description of this profile
    pub description: Option<String>,

    /// Bootstrap script to run for this profile
    pub bootstrap_script: Option<String>,

    /// Files to include in this profile
    #[serde(default)]
    pub files: Vec<String>,

    /// Files to exclude from this profile
    #[serde(default)]
    pub exclude: Vec<String>,

    /// System commands to generate (but not execute)
    #[serde(default)]
    pub system_commands: Vec<String>,

    /// Homebrew packages to install
    #[serde(default)]
    pub homebrew_packages: Vec<String>,

    /// VS Code extensions to install
    #[serde(default)]
    pub vscode_extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Age key file path
    pub age_key_file: Option<String>,

    /// SOPS configuration file
    pub sops_config: Option<String>,

    /// Encrypted files patterns
    #[serde(default)]
    pub encrypted_patterns: Vec<String>,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            default_profile: default_profile(),
            auto_push: false,
            backup_existing: default_backup(),
        }
    }
}

impl Default for SecretsConfig {
    fn default() -> Self {
        Self {
            age_key_file: None,
            sops_config: None,
            encrypted_patterns: vec!["*.enc.yaml".to_string(), "*.enc.yml".to_string()],
        }
    }
}

fn default_profile() -> String {
    "default".to_string()
}

fn default_backup() -> bool {
    true
}

#[allow(dead_code)]
impl Config {
    /// Load configuration from a TOML file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to a TOML file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get a specific profile configuration
    pub fn get_profile(&self, name: &str) -> Option<&ProfileConfig> {
        self.profiles.get(name)
    }

    /// Get the default profile configuration
    pub fn get_default_profile(&self) -> Option<&ProfileConfig> {
        self.get_profile(&self.global.default_profile)
    }

    /// Create a new profile
    pub fn add_profile(&mut self, name: String, config: ProfileConfig) {
        self.profiles.insert(name, config);
    }

    /// Remove a profile
    pub fn remove_profile(&mut self, name: &str) -> Option<ProfileConfig> {
        self.profiles.remove(name)
    }

    /// List all profile names
    pub fn list_profiles(&self) -> Vec<&String> {
        self.profiles.keys().collect()
    }
}

#[allow(dead_code)]
impl ProfileConfig {
    /// Create a new profile configuration
    pub fn new(description: Option<String>) -> Self {
        Self {
            description,
            bootstrap_script: None,
            files: Vec::new(),
            exclude: Vec::new(),
            system_commands: Vec::new(),
            homebrew_packages: Vec::new(),
            vscode_extensions: Vec::new(),
        }
    }

    /// Add a file to this profile
    pub fn add_file(&mut self, file: String) {
        if !self.files.contains(&file) {
            self.files.push(file);
        }
    }

    /// Remove a file from this profile
    pub fn remove_file(&mut self, file: &str) {
        self.files.retain(|f| f != file);
    }

    /// Add a system command to this profile
    pub fn add_system_command(&mut self, command: String) {
        if !self.system_commands.contains(&command) {
            self.system_commands.push(command);
        }
    }

    /// Add a Homebrew package to this profile
    pub fn add_homebrew_package(&mut self, package: String) {
        if !self.homebrew_packages.contains(&package) {
            self.homebrew_packages.push(package);
        }
    }

    /// Add a VS Code extension to this profile
    pub fn add_vscode_extension(&mut self, extension: String) {
        if !self.vscode_extensions.contains(&extension) {
            self.vscode_extensions.push(extension);
        }
    }
}
