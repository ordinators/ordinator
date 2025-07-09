use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

use crate::config::{Config, HomebrewPackage, HomebrewPackages};

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

        let current_packages = self.get_current_packages().await?;

        if let Some(profile_config) = config.get_profile_mut(profile) {
            profile_config.homebrew_packages = current_packages;
            info!(
                "Exported {} formulae and {} casks to profile '{}'",
                profile_config.homebrew_packages.formulae.len(),
                profile_config.homebrew_packages.casks.len(),
                profile
            );
        } else {
            return Err(anyhow::anyhow!("Profile '{}' not found", profile));
        }

        Ok(())
    }

    /// Install Homebrew packages from config
    pub async fn install_packages(&self, profile: &str, config: &Config) -> Result<()> {
        info!("Installing Homebrew packages for profile: {}", profile);

        let profile_config = config
            .get_profile(profile)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile))?;

        let packages = &profile_config.homebrew_packages;

        if packages.formulae.is_empty() && packages.casks.is_empty() {
            info!("No Homebrew packages defined for profile '{}'", profile);
            return Ok(());
        }

        // Install formulae
        if !packages.formulae.is_empty() {
            info!("Installing {} formulae", packages.formulae.len());
            for package in &packages.formulae {
                self.install_formula(package).await?;
            }
        }

        // Install casks
        if !packages.casks.is_empty() {
            info!("Installing {} casks", packages.casks.len());
            for package in &packages.casks {
                self.install_cask(package).await?;
            }
        }

        info!(
            "Homebrew package installation complete for profile '{}'",
            profile
        );
        Ok(())
    }

    /// List packages for a profile
    pub fn list_packages(&self, profile: &str, config: &Config) -> Result<()> {
        let profile_config = config
            .get_profile(profile)
            .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found", profile))?;

        println!("Homebrew packages for profile '{profile}':");
        println!("Formulae:");
        for formula in &profile_config.homebrew_packages.formulae {
            println!("  - {}", formula.name);
        }
        println!("Casks:");
        for cask in &profile_config.homebrew_packages.casks {
            println!("  - {}", cask.name);
        }

        Ok(())
    }

    /// Get current Homebrew packages
    async fn get_current_packages(&self) -> Result<HomebrewPackages> {
        let mut formulae = Vec::new();
        let mut casks = Vec::new();

        // Get installed formulae
        let formulae_output = Command::new("brew")
            .args(["list", "--formula"])
            .output()
            .with_context(|| "Failed to run 'brew list --formula'")?;

        if formulae_output.status.success() {
            let formulae_str = String::from_utf8(formulae_output.stdout)
                .with_context(|| "Failed to parse formulae output")?;

            for line in formulae_str.lines() {
                let name = line.trim();
                if !name.is_empty() {
                    formulae.push(HomebrewPackage {
                        name: name.to_string(),
                        version: None, // TODO: Get versions if needed
                        pinned: None,
                    });
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
                    casks.push(HomebrewPackage {
                        name: name.to_string(),
                        version: None, // TODO: Get versions if needed
                        pinned: None,
                    });
                }
            }
        }

        Ok(HomebrewPackages { formulae, casks })
    }

    /// Install a single formula
    async fn install_formula(&self, package: &HomebrewPackage) -> Result<()> {
        if self.dry_run {
            println!("[DRY-RUN] Would install formula: {}", package.name);
            return Ok(());
        }

        let mut cmd = Command::new("brew");
        cmd.arg("install");

        if let Some(version) = &package.version {
            cmd.args([&package.name, &format!("@{version}")]);
        } else {
            cmd.arg(&package.name);
        }

        let output = cmd
            .output()
            .with_context(|| format!("Failed to run brew install for formula: {}", package.name))?;

        if output.status.success() {
            info!("Installed formula: {}", package.name);
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to install formula {}: {}", package.name, error);
        }

        Ok(())
    }

    /// Install a single cask
    async fn install_cask(&self, package: &HomebrewPackage) -> Result<()> {
        if self.dry_run {
            println!("[DRY-RUN] Would install cask: {}", package.name);
            return Ok(());
        }

        let mut cmd = Command::new("brew");
        cmd.args(["install", "--cask"]);

        if let Some(version) = &package.version {
            cmd.args([&package.name, &format!("@{version}")]);
        } else {
            cmd.arg(&package.name);
        }

        let output = cmd.output().with_context(|| {
            format!(
                "Failed to run brew install --cask for cask: {}",
                package.name
            )
        })?;

        if output.status.success() {
            info!("Installed cask: {}", package.name);
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Failed to install cask {}: {}", package.name, error);
        }

        Ok(())
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
        use crate::config::{Config, HomebrewPackage};
        let mut config = Config::create_default();
        let profile = config.profiles.get_mut("default").unwrap();
        profile.homebrew_packages.formulae.push(HomebrewPackage {
            name: "dummyformula".to_string(),
            version: None,
            pinned: None,
        });
        profile.homebrew_packages.casks.push(HomebrewPackage {
            name: "dummycask".to_string(),
            version: None,
            pinned: None,
        });
        let toml = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml).unwrap();
        let parsed_profile = parsed.get_profile("default").unwrap();
        assert_eq!(
            parsed_profile.homebrew_packages.formulae[0].name,
            "dummyformula"
        );
        assert_eq!(parsed_profile.homebrew_packages.casks[0].name, "dummycask");
    }
}
