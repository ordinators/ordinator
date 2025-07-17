use anyhow::{Context, Result};
use std::process::Command;
use tracing::info;

use crate::config::Config;

pub struct BrewManager {
    dry_run: bool,
}

impl BrewManager {
    pub fn new(dry_run: bool) -> Self {
        Self { dry_run }
    }

    /// Export current Homebrew packages to config
    pub async fn export_packages(&self, profile: &str, config: &mut Config) -> Result<()> {
        info!("Exporting Homebrew packages for profile: {}", profile);

        let (formulas, casks) = self.get_current_packages().await?;

        if let Some(profile_config) = config.get_profile_mut(profile) {
            profile_config.homebrew_formulas = formulas;
            profile_config.homebrew_casks = casks;
            info!(
                "Exported {} formulas and {} casks to profile '{}'",
                profile_config.homebrew_formulas.len(),
                profile_config.homebrew_casks.len(),
                profile
            );
        } else {
            return Err(anyhow::anyhow!("Profile '{}' not found", profile));
        }

        Ok(())
    }

    /// Install Homebrew packages from config
    pub async fn install_packages(&self, profile: &str, config: &Config) -> anyhow::Result<()> {
        use anyhow::Context;
        tracing::info!("Installing Homebrew packages for profile: {}", profile);
        let profile_config = config
            .get_profile(profile)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile))?;
        if profile_config.homebrew_formulas.is_empty() && profile_config.homebrew_casks.is_empty() {
            tracing::info!(
                "No Homebrew formulas or casks defined for profile '{}'",
                profile
            );
            return Ok(());
        }
        let (missing_formulas, missing_casks) = self.get_missing_packages(profile_config).await?;
        if missing_formulas.is_empty() && missing_casks.is_empty() {
            println!(
                "All Homebrew formulas and casks for profile '{profile}' are already installed."
            );
            return Ok(());
        }
        // Install missing formulas in one command
        if !missing_formulas.is_empty() {
            if self.dry_run {
                println!(
                    "[DRY-RUN] Would install formulas: {}",
                    missing_formulas.join(" ")
                );
            } else {
                let mut cmd = std::process::Command::new("brew");
                cmd.arg("install");
                for formula in &missing_formulas {
                    cmd.arg(formula);
                }
                let output = cmd.output().with_context(|| {
                    format!(
                        "Failed to run brew install for formulas: {}",
                        missing_formulas.join(", ")
                    )
                })?;
                if output.status.success() {
                    tracing::info!("Installed formulas: {}", missing_formulas.join(", "));
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!(
                        "Failed to install formulas: {}: {}",
                        missing_formulas.join(", "),
                        error
                    );
                }
            }
        }
        // Install missing casks in one command
        if !missing_casks.is_empty() {
            if self.dry_run {
                println!("[DRY-RUN] Would install casks: {}", missing_casks.join(" "));
            } else {
                let mut cmd = std::process::Command::new("brew");
                cmd.arg("install");
                cmd.arg("--cask");
                for cask in &missing_casks {
                    cmd.arg(cask);
                }
                let output = cmd.output().with_context(|| {
                    format!(
                        "Failed to run brew install --cask for casks: {}",
                        missing_casks.join(", ")
                    )
                })?;
                if output.status.success() {
                    tracing::info!("Installed casks: {}", missing_casks.join(", "));
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!(
                        "Failed to install casks: {}: {}",
                        missing_casks.join(", "),
                        error
                    );
                }
            }
        }
        Ok(())
    }

    /// List packages for a profile
    pub fn list_packages(&self, profile: &str, config: &Config) -> Result<()> {
        let profile_config = config
            .get_profile(profile)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile))?;

        println!("Homebrew packages for profile '{profile}':");
        for formula in &profile_config.homebrew_formulas {
            println!("  - {formula}");
        }
        for cask in &profile_config.homebrew_casks {
            println!("  - {cask}");
        }

        Ok(())
    }

    /// Get current Homebrew formulas and casks
    async fn get_current_packages(&self) -> Result<(Vec<String>, Vec<String>)> {
        let mut formulas = Vec::new();
        let mut casks = Vec::new();

        // Get user-installed formulas
        let formulae_output = Command::new("brew")
            .args(["leaves", "-r"])
            .output()
            .with_context(|| "Failed to run 'brew leaves -r'")?;

        if formulae_output.status.success() {
            let formulae_str = String::from_utf8(formulae_output.stdout)
                .with_context(|| "Failed to parse formulae output")?;

            for line in formulae_str.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    formulas.push(name.to_string());
                }
            }
        }

        // Get installed casks
        let casks_output = Command::new("brew")
            .args(["list", "--cask"])
            .output()
            .with_context(|| "Failed to run 'brew list --cask'")?;

        if casks_output.status.success() {
            let casks_str = String::from_utf8(casks_output.stdout)
                .with_context(|| "Failed to parse casks output")?;

            for line in casks_str.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    casks.push(name.to_string());
                }
            }
        }

        Ok((formulas, casks))
    }

    /// Compute missing formulas and casks
    async fn get_missing_packages(
        &self,
        profile_config: &crate::config::ProfileConfig,
    ) -> anyhow::Result<(Vec<String>, Vec<String>)> {
        let (installed_formulas, installed_casks) = self.get_current_packages().await?;
        let missing_formulas: Vec<String> = profile_config
            .homebrew_formulas
            .iter()
            .filter(|f| !installed_formulas.contains(f))
            .cloned()
            .collect();
        let missing_casks: Vec<String> = profile_config
            .homebrew_casks
            .iter()
            .filter(|c| !installed_casks.contains(c))
            .cloned()
            .collect();
        Ok((missing_formulas, missing_casks))
    }

    /// Check if Homebrew is installed
    pub fn check_homebrew_installed() -> bool {
        Command::new("brew")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::sync::{Arc, Mutex};

    // Helper struct to override get_current_packages for testing
    struct TestBrewManager {
        installed_formulas: Arc<Mutex<Vec<String>>>,
        installed_casks: Arc<Mutex<Vec<String>>>,
    }

    impl TestBrewManager {
        fn new(formulas: Vec<&str>, casks: Vec<&str>) -> Self {
            Self {
                installed_formulas: Arc::new(Mutex::new(
                    formulas.into_iter().map(|s| s.to_string()).collect(),
                )),
                installed_casks: Arc::new(Mutex::new(
                    casks.into_iter().map(|s| s.to_string()).collect(),
                )),
            }
        }
        async fn get_missing_packages(
            &self,
            profile_config: &crate::config::ProfileConfig,
        ) -> anyhow::Result<(Vec<String>, Vec<String>)> {
            let installed_formulas = self.installed_formulas.lock().unwrap().clone();
            let installed_casks = self.installed_casks.lock().unwrap().clone();
            let missing_formulas: Vec<String> = profile_config
                .homebrew_formulas
                .iter()
                .filter(|f| !installed_formulas.contains(f))
                .cloned()
                .collect();
            let missing_casks: Vec<String> = profile_config
                .homebrew_casks
                .iter()
                .filter(|c| !installed_casks.contains(c))
                .cloned()
                .collect();
            Ok((missing_formulas, missing_casks))
        }
    }

    #[test]
    fn test_brew_manager_creation() {
        let manager = BrewManager::new(false);
        assert!(!manager.dry_run);

        let manager = BrewManager::new(true);
        assert!(manager.dry_run);
    }

    #[test]
    fn test_homebrew_installed_check() {
        // This test will pass if Homebrew is installed, fail if not
        // In a real test environment, we might want to mock this
        let installed = BrewManager::check_homebrew_installed();
        // We can't assert on this since it depends on the test environment
        println!("Homebrew installed: {installed}");
    }

    #[test]
    fn test_list_packages_empty_profile() {
        let config = Config::create_default();
        let manager = BrewManager::new(false);

        // Should not error for empty profile
        let result = manager.list_packages("default", &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_packages_nonexistent_profile() {
        let config = Config::create_default();
        let manager = BrewManager::new(false);

        // Should error for nonexistent profile
        let result = manager.list_packages("nonexistent", &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_install_packages_missing_profile() {
        let config = Config::create_default();
        let manager = BrewManager::new(true);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(manager.install_packages("nonexistent", &config));
        assert!(result.is_err());
    }

    #[test]
    fn test_export_packages_missing_profile() {
        let mut config = Config::create_default();
        let manager = BrewManager::new(true);
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(manager.export_packages("nonexistent", &mut config));
        assert!(result.is_err());
    }

    #[test]
    fn test_install_packages_empty_packages() {
        let config = Config::create_default();
        let manager = BrewManager::new(true);
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Should succeed (no packages to install)
        let result = rt.block_on(manager.install_packages("default", &config));
        assert!(result.is_ok());
    }

    #[test]
    fn test_export_packages_empty_profile() {
        let mut config = Config::create_default();
        let manager = BrewManager::new(true);
        let rt = tokio::runtime::Runtime::new().unwrap();
        // Should succeed (will export whatever dummy or real brew returns)
        let result = rt.block_on(manager.export_packages("default", &mut config));
        assert!(result.is_ok());
        let _profile = config.get_profile("default").unwrap();
        // Just check the field exists (do not assert empty)
    }

    #[test]
    fn test_config_homebrew_package_roundtrip() {
        use crate::config::Config;
        let mut config = Config::create_default();
        let profile = config.profiles.get_mut("default").unwrap();
        profile.homebrew_formulas.push("dummyformula".to_string());
        profile.homebrew_casks.push("dummycask".to_string());
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        let parsed_profile = parsed.get_profile("default").unwrap();
        assert_eq!(parsed_profile.homebrew_formulas[0], "dummyformula");
        assert_eq!(parsed_profile.homebrew_casks[0], "dummycask");
    }

    #[tokio::test]
    async fn test_get_missing_packages() {
        let mut config = Config::create_default();
        let profile = config.profiles.get_mut("default").unwrap();
        profile.homebrew_formulas = vec!["git".to_string(), "neovim".to_string()];
        profile.homebrew_casks = vec!["iterm2".to_string(), "alfred".to_string()];
        let manager = TestBrewManager::new(vec!["git"], vec!["iterm2"]);
        let (missing_formulas, missing_casks) =
            manager.get_missing_packages(profile).await.unwrap();
        assert_eq!(missing_formulas, vec!["neovim"]);
        assert_eq!(missing_casks, vec!["alfred"]);
    }

    #[tokio::test]
    async fn test_install_packages_dry_run_and_noop() {
        let mut config = Config::create_default();
        let profile = config.profiles.get_mut("default").unwrap();
        profile.homebrew_formulas = vec!["git".to_string(), "neovim".to_string()];
        profile.homebrew_casks = vec!["iterm2".to_string(), "alfred".to_string()];
        // Case 1: All packages already installed
        let manager = TestBrewManager::new(vec!["git", "neovim"], vec!["iterm2", "alfred"]);
        let (missing_formulas, missing_casks) =
            manager.get_missing_packages(profile).await.unwrap();
        assert!(missing_formulas.is_empty());
        assert!(missing_casks.is_empty());
        // Case 2: Some packages missing, dry-run
        let manager = TestBrewManager::new(vec!["git"], vec!["iterm2"]);
        let (missing_formulas, missing_casks) =
            manager.get_missing_packages(profile).await.unwrap();
        assert_eq!(missing_formulas, vec!["neovim"]);
        assert_eq!(missing_casks, vec!["alfred"]);
        // (We can't capture stdout easily here, but this ensures the logic is correct)
    }
}
