use crate::config::Config;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::fs;
#[cfg(test)]
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use tracing::info;
use walkdir::WalkDir;

/// Secrets manager using SOPS and age
#[allow(dead_code)]
pub struct SecretsManager {
    age_key_file: Option<std::path::PathBuf>,
    sops_config: Option<std::path::PathBuf>,
    config: Config,
    base_dir: std::path::PathBuf,
    encrypt_patterns: Option<GlobSet>,
    exclude_patterns: Option<GlobSet>,
}

impl SecretsManager {
    /// Create a new secrets manager
    #[allow(dead_code)]
    pub fn new(
        age_key_file: Option<std::path::PathBuf>,
        sops_config: Option<std::path::PathBuf>,
        config: Config,
        base_dir: std::path::PathBuf,
    ) -> Self {
        Self {
            age_key_file,
            sops_config,
            config,
            base_dir,
            encrypt_patterns: None,
            exclude_patterns: None,
        }
    }

    /// Create a GlobSet from patterns
    fn create_glob_set(patterns: &[String]) -> Result<Option<GlobSet>> {
        if patterns.is_empty() {
            return Ok(None);
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern)?);
        }
        Ok(Some(builder.build()?))
    }

    /// Check if a file should be encrypted based on patterns
    fn should_encrypt_file(&mut self, file_path: &Path) -> Result<bool> {
        let file_str = file_path.to_string_lossy();

        // Create glob sets if they don't exist
        if self.encrypt_patterns.is_none() {
            self.encrypt_patterns = Self::create_glob_set(&self.config.secrets.encrypt_patterns)?;
        }
        if self.exclude_patterns.is_none() {
            self.exclude_patterns = Self::create_glob_set(&self.config.secrets.exclude_patterns)?;
        }

        // Check if file matches any encrypt pattern
        if let Some(encrypt_patterns) = &self.encrypt_patterns {
            if encrypt_patterns.is_match(&*file_str) {
                // Check if file matches any exclude pattern
                if let Some(exclude_patterns) = &self.exclude_patterns {
                    if exclude_patterns.is_match(&*file_str) {
                        return Ok(false);
                    }
                }
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Encrypt a file using SOPS
    #[allow(dead_code)]
    pub fn encrypt_file(&mut self, file_path: &Path) -> Result<()> {
        info!("Encrypting file: {:?}", file_path);

        // Check if file should be encrypted based on patterns
        if !self.should_encrypt_file(file_path)? {
            info!("File {:?} does not match encryption patterns", file_path);
            return Ok(());
        }

        // Call the actual encryption function
        encrypt_file_with_sops(file_path.to_str().unwrap())?;
        Ok(())
    }

    /// Decrypt a file using SOPS
    #[allow(dead_code)]
    pub fn decrypt_file(&mut self, file_path: &Path) -> Result<()> {
        info!("Decrypting file: {:?}", file_path);

        // Check if file should be decrypted based on patterns
        if !self.should_encrypt_file(file_path)? {
            info!("File {:?} does not match encryption patterns", file_path);
            return Ok(());
        }

        // Call the actual decryption function
        decrypt_file_with_sops(file_path.to_str().unwrap())?;
        Ok(())
    }

    /// List encrypted files in the repository
    pub fn list_encrypted_files(&self) -> anyhow::Result<Vec<(std::path::PathBuf, bool)>> {
        let encrypt_patterns = &self.config.secrets.encrypt_patterns;
        let exclude_patterns = &self.config.secrets.exclude_patterns;
        if encrypt_patterns.is_empty() {
            return Ok(vec![]);
        }
        let mut encrypt_builder = GlobSetBuilder::new();
        for pat in encrypt_patterns {
            encrypt_builder.add(Glob::new(pat)?);
        }
        let encrypt_set = encrypt_builder.build()?;
        let mut exclude_set = None;
        if !exclude_patterns.is_empty() {
            let mut builder = GlobSetBuilder::new();
            for pat in exclude_patterns {
                builder.add(Glob::new(pat)?);
            }
            exclude_set = Some(builder.build()?);
        }
        let mut results = vec![];
        let base_dir = &self.base_dir;
        for entry in WalkDir::new(base_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let rel = path.strip_prefix(base_dir).unwrap_or(path);
                let rel_str = rel.to_string_lossy();
                let encrypt_match = encrypt_set.is_match(&*rel_str);
                let exclude_match = exclude_set
                    .as_ref()
                    .map(|ex| ex.is_match(&*rel_str))
                    .unwrap_or(false);
                if encrypt_match && !exclude_match {
                    let encrypted = is_file_encrypted(path);
                    results.push((rel.to_path_buf(), encrypted));
                }
            }
        }
        Ok(results)
    }

    /// Check if a file contains plaintext secrets
    #[allow(dead_code)]
    pub fn check_for_plaintext_secrets(&self, _file_path: &std::path::Path) -> Result<bool> {
        // TODO: Implement plaintext secrets detection
        Ok(false)
    }

    /// Validate SOPS and age installation
    #[allow(dead_code)]
    pub fn validate_installation(&self) -> Result<()> {
        // TODO: Check if SOPS and age are installed
        Ok(())
    }
}

pub fn check_sops_and_age() -> anyhow::Result<()> {
    let sops = which::which("sops").map_err(|_| anyhow::anyhow!(
        "SOPS is not installed or not found in PATH.\nInstall it: https://github.com/mozilla/sops#downloads"
    ))?;
    let age = which::which("age").map_err(|_| anyhow::anyhow!(
        "age is not installed or not found in PATH.\nInstall it: https://github.com/FiloSottile/age#installation"
    ))?;
    println!("Found sops at: {}", sops.display());
    println!("Found age at: {}", age.display());
    Ok(())
}

/// Set up SOPS and age for secrets management
pub fn setup_sops_and_age(profile: &str, force: bool) -> anyhow::Result<()> {
    info!("Setting up SOPS and age for profile: {}", profile);
    match check_sops_and_age() {
        Ok(()) => {
            println!("✅ SOPS and age are already installed");
        }
        Err(_) => {
            println!("❌ SOPS and/or age not found. Installing via Homebrew...");
            install_sops_and_age()?;
        }
    }
    let config_base = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("ordinator");
    let age_key_path = generate_age_key(&config_base, profile, force)?;
    let sops_config_path = create_sops_config(profile, &age_key_path, force)?;
    update_ordinator_config(profile, &age_key_path, &sops_config_path)?;
    println!("✅ SOPS and age setup complete for profile: {profile}");
    println!("   Age key: {}", age_key_path.display());
    println!("   SOPS config: {}", sops_config_path.display());
    Ok(())
}

/// Install SOPS and age via Homebrew
fn install_sops_and_age() -> anyhow::Result<()> {
    println!("Installing SOPS and age via Homebrew...");

    // Check if Homebrew is installed
    if which::which("brew").is_err() {
        return Err(anyhow::anyhow!(
            "Homebrew is not installed. Please install Homebrew first: https://brew.sh"
        ));
    }

    // Install SOPS and age
    let status = Command::new("brew")
        .args(["install", "sops", "age"])
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!(
            "Failed to install SOPS and age via Homebrew"
        ));
    }

    println!("✅ SOPS and age installed successfully");
    Ok(())
}

/// Generate an age key for the specified profile
fn generate_age_key(
    base_dir: &Path,
    profile: &str,
    force: bool,
) -> anyhow::Result<std::path::PathBuf> {
    let config_dir = base_dir.join("age");
    fs::create_dir_all(&config_dir)?;
    let key_filename = if profile == "default" {
        "key.txt".to_string()
    } else {
        format!("{profile}.txt")
    };
    let key_path = config_dir.join(key_filename);
    if let Some(parent) = key_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if key_path.exists() && force {
        fs::remove_file(&key_path)?;
    }
    if key_path.exists() && !force {
        println!("✅ Age key already exists: {}", key_path.display());
        return Ok(key_path);
    }
    println!("Generating age key for profile: {profile}");
    let output = Command::new("age-keygen")
        .arg("-o")
        .arg(&key_path)
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to generate age key: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600))?;
    println!("✅ Age key generated: {}", key_path.display());
    Ok(key_path)
}

/// Create SOPS configuration file
fn create_sops_config(
    profile: &str,
    age_key_path: &Path,
    force: bool,
) -> anyhow::Result<std::path::PathBuf> {
    let config_filename = if profile == "default" {
        ".sops.yaml".to_string()
    } else {
        format!(".sops.{profile}.yaml")
    };

    let sops_config_path = std::env::current_dir()?.join(&config_filename);

    if sops_config_path.exists() && !force {
        println!(
            "✅ SOPS config already exists: {}",
            sops_config_path.display()
        );
        return Ok(sops_config_path);
    }

    println!("Creating SOPS configuration for profile: {profile}");

    // Read the age public key
    let age_key_content = fs::read_to_string(age_key_path)?;

    // Debug: Write all lines to a temporary file for inspection
    let debug_file = std::env::temp_dir().join("age_key_debug.txt");
    fs::write(&debug_file, &age_key_content)
        .map_err(|e| anyhow::anyhow!("Failed to write debug file: {}", e))?;
    println!(
        "Debug: Age key file contents written to {}",
        debug_file.display()
    );

    let public_key = age_key_content
        .lines()
        .find_map(|line| {
            if line.starts_with("# public key: ") {
                Some(line.trim_start_matches("# public key: ").trim())
            } else if line.starts_with("age1") {
                Some(line.trim())
            } else {
                None
            }
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find age public key in key file. Debug file: {:?}",
                debug_file
            )
        })?;

    // Create SOPS configuration
    let sops_config = format!(r#"creation_rules:\n  - age: >-\n      {public_key}\n"#);

    fs::write(&sops_config_path, sops_config)?;

    println!("✅ SOPS config created: {}", sops_config_path.display());
    Ok(sops_config_path)
}

/// Update ordinator.toml with secrets configuration
fn update_ordinator_config(
    _profile: &str,
    age_key_path: &Path,
    sops_config_path: &Path,
) -> anyhow::Result<()> {
    let config_path = crate::config::Config::find_config_file()?
        .ok_or_else(|| anyhow::anyhow!("Could not find ordinator.toml configuration file"))?;
    let mut config = if config_path.exists() {
        Config::from_file(&config_path)?
    } else {
        Config::default()
    };

    // Update secrets configuration
    config.secrets.age_key_file = Some(age_key_path.to_path_buf());
    config.secrets.sops_config = Some(sops_config_path.to_path_buf());

    // Add default encryption patterns if none exist
    if config.secrets.encrypt_patterns.is_empty() {
        config.secrets.encrypt_patterns = vec![
            "secrets/**/*.yaml".to_string(),
            "secrets/**/*.yml".to_string(),
            "*.key".to_string(),
        ];
    }

    // Add default exclude patterns if none exist
    if config.secrets.exclude_patterns.is_empty() {
        config.secrets.exclude_patterns = vec!["*.bak".to_string(), "**/*.enc.yaml".to_string()];
    }

    // Save updated config
    config.save_to_file(&config_path)?;

    println!("✅ Updated ordinator.toml with secrets configuration");
    Ok(())
}

pub fn encrypt_file_with_sops(file: &str) -> anyhow::Result<String> {
    use std::path::Path;
    use std::process::Command;

    // Check if sops is available
    check_sops_and_age()?;

    // Load configuration to get age key file
    let config = crate::config::Config::from_file_or_default()?;
    let age_key_file = config.secrets.age_key_file.ok_or_else(|| {
        anyhow::anyhow!("No age key file configured. Run 'ordinator secrets setup' first.")
    })?;

    let input_path = Path::new(file);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }
    let file = input_path.to_string_lossy().to_string();
    let file_name = input_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("file");

    // Determine output path - always relative to input file's directory
    let output_path = if let Some(ext) = input_path.extension().and_then(|e| e.to_str()) {
        if ext == "yaml" || ext == "yml" {
            let stem = input_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(file_name);
            let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
            parent
                .join(format!("{stem}.enc.{ext}"))
                .to_string_lossy()
                .to_string()
        } else {
            let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
            parent
                .join(format!("{file_name}.enc"))
                .to_string_lossy()
                .to_string()
        }
    } else {
        let parent = input_path.parent().unwrap_or_else(|| Path::new(""));
        parent
            .join(format!("{file_name}.enc"))
            .to_string_lossy()
            .to_string()
    };

    // Call sops to encrypt with age key file set
    let mut command = Command::new("sops");
    command
        .arg("--encrypt")
        .arg(&file)
        .arg("--output")
        .arg(&output_path)
        .env("SOPS_AGE_KEY_FILE", age_key_file);

    let status = command.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("sops failed to encrypt file: {}", file));
    }
    info!("Successfully encrypted file: {} to {}", file, output_path);
    Ok(output_path)
}

pub fn decrypt_file_with_sops(file: &str) -> anyhow::Result<()> {
    use std::path::Path;
    use std::process::Command;

    // Check if sops is available
    check_sops_and_age()?;

    // Load configuration to get age key file
    let config = crate::config::Config::from_file_or_default()?;
    let age_key_file = config.secrets.age_key_file.ok_or_else(|| {
        anyhow::anyhow!("No age key file configured. Run 'ordinator secrets setup' first.")
    })?;

    let input_path = Path::new(file);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }

    // Call sops to decrypt with age key file set
    let mut command = Command::new("sops");
    command
        .arg("--decrypt")
        .arg(file)
        .env("SOPS_AGE_KEY_FILE", age_key_file);

    let status = command.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("sops failed to decrypt file: {}", file));
    }
    info!("Successfully decrypted file: {}", file);
    Ok(())
}

/// Returns true if the file appears to be SOPS-encrypted (by header)
pub fn is_file_encrypted(path: &std::path::Path) -> bool {
    if let Ok(file) = fs::File::open(path) {
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            if let Ok(l) = line {
                if l.trim().starts_with("sops:") {
                    return true;
                }
            }
            if i > 10 {
                break;
            }
        }
    }
    false
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    /// Test isolation guard that ensures complete isolation and automatic cleanup
    struct TestIsolationGuard {
        temp_dir: tempfile::TempDir,
        orig_config: Option<String>,
        orig_home: Option<String>,
    }

    impl TestIsolationGuard {
        /// Create a new test isolation environment with a unique temp directory
        fn new() -> Self {
            let temp_dir = tempdir().unwrap();
            let orig_config = env::var("ORDINATOR_CONFIG").ok();
            let orig_home = env::var("ORDINATOR_HOME").ok();

            // Set up completely isolated environment
            env::set_var("ORDINATOR_HOME", temp_dir.path());
            // Don't change working directory globally to avoid thread conflicts

            Self {
                temp_dir,
                orig_config,
                orig_home,
            }
        }

        /// Return the path to the temp dir
        pub fn temp_dir(&self) -> &tempfile::TempDir {
            &self.temp_dir
        }
    }

    impl Drop for TestIsolationGuard {
        fn drop(&mut self) {
            // Clean up environment variables
            if let Some(config_val) = &self.orig_config {
                env::set_var("ORDINATOR_CONFIG", config_val);
            } else {
                env::remove_var("ORDINATOR_CONFIG");
            }
            if let Some(home_val) = &self.orig_home {
                env::set_var("ORDINATOR_HOME", home_val);
            } else {
                env::remove_var("ORDINATOR_HOME");
            }
            // Don't restore working directory to avoid thread conflicts
        }
    }

    #[test]
    fn test_encrypt_file() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let mut manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        let file_path = _guard.temp_dir().path().join("test.txt");
        let result = manager.encrypt_file(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_file_with_configs() {
        let _guard = TestIsolationGuard::new();
        let age_key = _guard.temp_dir().path().join("age.key");
        let sops_config = _guard.temp_dir().path().join(".sops.yaml");
        let config = Config::default();
        let mut manager = SecretsManager::new(
            Some(age_key),
            Some(sops_config),
            config,
            _guard.temp_dir().path().to_path_buf(),
        );
        let file_path = _guard.temp_dir().path().join("test.txt");
        let result = manager.encrypt_file(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_file() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let mut manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        let file_path = _guard.temp_dir().path().join("test.enc.yaml");
        let result = manager.decrypt_file(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_file_with_configs() {
        let _guard = TestIsolationGuard::new();
        let age_key = _guard.temp_dir().path().join("age.key");
        let sops_config = _guard.temp_dir().path().join(".sops.yaml");
        let config = Config::default();
        let mut manager = SecretsManager::new(
            Some(age_key),
            Some(sops_config),
            config,
            _guard.temp_dir().path().to_path_buf(),
        );
        let file_path = _guard.temp_dir().path().join("test.enc.yaml");
        let result = manager.decrypt_file(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_encrypted_files() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        let _repo_path = _guard.temp_dir().path().join("repo");
        let result = manager.list_encrypted_files();
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.is_empty()); // Currently returns empty vector
    }

    #[test]
    fn test_list_encrypted_files_with_configs() {
        let _guard = TestIsolationGuard::new();
        let age_key = _guard.temp_dir().path().join("age.key");
        let sops_config = _guard.temp_dir().path().join(".sops.yaml");
        let config = Config::default();
        let manager = SecretsManager::new(
            Some(age_key),
            Some(sops_config),
            config,
            _guard.temp_dir().path().to_path_buf(),
        );
        let _repo_path = _guard.temp_dir().path().join("repo");
        let result = manager.list_encrypted_files();
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_check_for_plaintext_secrets() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        let file_path = _guard.temp_dir().path().join("test.txt");
        let result = manager.check_for_plaintext_secrets(&file_path);
        assert!(result.is_ok());
        let has_secrets = result.unwrap();
        assert!(!has_secrets); // Currently returns false
    }

    #[test]
    fn test_check_for_plaintext_secrets_with_configs() {
        let _guard = TestIsolationGuard::new();
        let age_key = _guard.temp_dir().path().join("age.key");
        let sops_config = _guard.temp_dir().path().join(".sops.yaml");
        let config = Config::default();
        let manager = SecretsManager::new(
            Some(age_key),
            Some(sops_config),
            config,
            _guard.temp_dir().path().to_path_buf(),
        );
        let file_path = _guard.temp_dir().path().join("test.txt");
        let result = manager.check_for_plaintext_secrets(&file_path);
        assert!(result.is_ok());
        let has_secrets = result.unwrap();
        assert!(!has_secrets); // Currently returns false
    }

    #[test]
    fn test_validate_installation() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        let result = manager.validate_installation();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_installation_with_configs() {
        let _guard = TestIsolationGuard::new();
        let age_key = _guard.temp_dir().path().join("age.key");
        let sops_config = _guard.temp_dir().path().join(".sops.yaml");
        // Create dummy age.key and .sops.yaml files
        std::fs::write(
            &age_key,
            "# public key: age1testkey\nAGE-SECRET-KEY-1TEST\n",
        )
        .unwrap();
        std::fs::write(&sops_config, "creation_rules: []\n").unwrap();
        let config = Config::default();
        let manager = SecretsManager::new(
            Some(age_key),
            Some(sops_config),
            config,
            _guard.temp_dir().path().to_path_buf(),
        );
        let result = manager.validate_installation();
        assert!(result.is_ok());
    }

    #[test]
    fn test_secrets_manager_integration() {
        let _guard = TestIsolationGuard::new();
        let age_key = _guard.temp_dir().path().join("age.key");
        let sops_config = _guard.temp_dir().path().join(".sops.yaml");
        let config = Config::default();
        let mut manager = SecretsManager::new(
            Some(age_key),
            Some(sops_config),
            config,
            _guard.temp_dir().path().to_path_buf(),
        );
        // Test all methods in sequence
        let file_path = _guard.temp_dir().path().join("test.txt");
        assert!(manager.encrypt_file(&file_path).is_ok());
        let enc_file_path = _guard.temp_dir().path().join("test.enc.yaml");
        assert!(manager.decrypt_file(&enc_file_path).is_ok());
        let _repo_path = _guard.temp_dir().path().join("repo");
        let files = manager.list_encrypted_files().unwrap();
        assert!(files.is_empty());
        assert!(manager.check_for_plaintext_secrets(&file_path).is_ok());
        assert!(manager.validate_installation().is_ok());
    }

    #[test]
    fn test_secrets_manager_edge_cases() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let mut manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());

        // Test with non-existent paths
        let non_existent_path = std::path::Path::new("/non/existent/path");
        assert!(manager.encrypt_file(non_existent_path).is_ok());
        assert!(manager.decrypt_file(non_existent_path).is_ok());
        assert!(manager.list_encrypted_files().is_ok());
        assert!(manager
            .check_for_plaintext_secrets(non_existent_path)
            .is_ok());
    }

    #[test]
    fn test_secrets_manager_with_empty_paths() {
        let _guard = TestIsolationGuard::new();
        let config = Config::default();
        let mut manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());

        // Test with empty path
        let empty_path = std::path::Path::new("");
        assert!(manager.encrypt_file(empty_path).is_ok());
        assert!(manager.decrypt_file(empty_path).is_ok());
        assert!(manager.list_encrypted_files().is_ok());
        assert!(manager.check_for_plaintext_secrets(empty_path).is_ok());
    }

    #[test]
    fn test_check_sops_and_age_not_found() {
        std::env::set_var("PATH", "");
        assert!(check_sops_and_age().is_err());
        std::env::remove_var("PATH");
    }

    #[test]
    fn test_check_sops_and_age_found() {
        let temp_dir = tempdir().unwrap();
        // Create dummy binaries
        let sops_path = temp_dir.path().join("sops");
        let age_path = temp_dir.path().join("age");

        // Create empty files with execute permissions
        std::fs::File::create(&sops_path).unwrap();
        std::fs::File::create(&age_path).unwrap();

        std::fs::set_permissions(&sops_path, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::set_permissions(&age_path, std::fs::Permissions::from_mode(0o755)).unwrap();

        // Add temp dir to PATH
        let orig_path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var(
            "PATH",
            format!("{}:{}", temp_dir.path().display(), orig_path),
        );

        assert!(check_sops_and_age().is_ok());

        // Restore original PATH
        let orig_path = orig_path.as_str();
        if std::path::Path::new(&orig_path).exists() {
            env::set_current_dir(orig_path).unwrap();
        }
    }

    #[test]
    fn test_encrypt_file_with_sops_file_not_found() {
        let file = "/non/existent/file.txt";
        assert!(encrypt_file_with_sops(file).is_err());
    }

    #[test]
    fn test_encrypt_file_with_sops_sops_not_found() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test").unwrap();

        std::env::set_var("PATH", "");
        assert!(encrypt_file_with_sops(file_path.to_str().unwrap()).is_err());
        std::env::remove_var("PATH");
    }

    #[test]
    fn test_encrypt_file_with_sops_failure() {
        use std::env;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        let temp_dir = tempdir().unwrap();
        let orig_path = env::var("PATH").unwrap_or_default();
        let orig_config = env::var("ORDINATOR_CONFIG").ok();

        // Create a dummy sops that always fails
        let sops_path = temp_dir.path().join("sops");
        fs::write(&sops_path, "#!/bin/sh\nexit 1\n").unwrap();
        fs::set_permissions(&sops_path, fs::Permissions::from_mode(0o755)).unwrap();
        // Create a dummy age binary
        let age_path = temp_dir.path().join("age");
        fs::write(&age_path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&age_path, fs::Permissions::from_mode(0o755)).unwrap();

        // Add temp dir to PATH
        let new_path = format!("{}:{}", temp_dir.path().display(), orig_path);
        std::env::set_var("PATH", &new_path);

        // Create a temp file to encrypt
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();

        // Ensure the parent directory exists
        let parent_dir = file_path.parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();

        let result = crate::secrets::encrypt_file_with_sops(file_path.to_str().unwrap());
        if let Err(e) = &result {
            println!("[TEST DEBUG] sops_failure error: {e}");
        }
        // The function should fail because sops exits with code 1
        assert!(
            result.is_err(),
            "Expected encryption to fail when sops exits with code 1"
        );

        // Clean up
        if let Some(config_val) = orig_config {
            env::set_var("ORDINATOR_CONFIG", config_val);
        } else {
            env::remove_var("ORDINATOR_CONFIG");
        }
        // Restore original PATH
        std::env::set_var("PATH", orig_path);
    }

    #[test]
    fn test_encrypt_file_with_sops_success() {
        use std::env;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::process::Command as ProcessCommand;
        let temp_dir = tempdir().unwrap();
        let orig_path = env::var("PATH").unwrap_or_default();
        let orig_config = env::var("ORDINATOR_CONFIG").ok();

        // Create a shell script as dummy sops that copies input to output
        let sops_path = temp_dir.path().join("sops");
        fs::write(
            &sops_path,
            "#!/bin/sh\nmkdir -p $(dirname \"$4\")\n/bin/cp \"$2\" \"$4\"\n",
        )
        .unwrap();
        fs::set_permissions(&sops_path, fs::Permissions::from_mode(0o755)).unwrap();
        // Create a dummy age binary
        let age_path = temp_dir.path().join("age");
        fs::write(&age_path, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&age_path, fs::Permissions::from_mode(0o755)).unwrap();

        // Add temp dir to PATH
        let new_path = format!("{}:{}", temp_dir.path().display(), orig_path);
        std::env::set_var("PATH", &new_path);

        // Create a dummy age key file
        let age_key_file = temp_dir.path().join("age.key");
        fs::write(
            &age_key_file,
            "# public key: age1testkey\nAGE-SECRET-KEY-1TEST\n",
        )
        .unwrap();

        // Create a config file with the age key file path
        let config = crate::config::Config {
            secrets: crate::config::SecretsConfig {
                age_key_file: Some(age_key_file.clone()),
                sops_config: None,
                encrypt_patterns: vec!["*.txt".to_string()],
                exclude_patterns: vec![],
            },
            ..Default::default()
        };
        let config_path = temp_dir.path().join("ordinator.toml");
        config.save_to_file(&config_path).unwrap();
        env::set_var("ORDINATOR_CONFIG", &config_path);

        // Set current working directory to temp dir to ensure relative paths work
        let orig_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create a temp file to encrypt
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "hello").unwrap();

        // Ensure the parent directory exists
        let parent_dir = file_path.parent().unwrap();
        fs::create_dir_all(parent_dir).unwrap();

        let result = crate::secrets::encrypt_file_with_sops(file_path.to_str().unwrap());
        if let Err(e) = &result {
            println!("[TEST DEBUG] sops_success error: {e}");
        }
        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(std::path::Path::new(&output_path).exists());
        let contents = fs::read_to_string(&output_path).unwrap();
        assert_eq!(contents, "hello");

        // Clean up
        if let Some(config_val) = orig_config {
            env::set_var("ORDINATOR_CONFIG", config_val);
        } else {
            env::remove_var("ORDINATOR_CONFIG");
        }
        // Restore original PATH and working directory
        std::env::set_var("PATH", orig_path);
        let _ = std::env::set_current_dir(orig_cwd);
    }

    #[test]
    fn test_is_file_encrypted_plaintext() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("plain.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "api_key: secret123").unwrap();
        assert!(!is_file_encrypted(&file_path));
    }

    #[test]
    fn test_is_file_encrypted_sops() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("secret.enc.yaml");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "sops:").unwrap();
        writeln!(file, "  kms: []").unwrap();
        assert!(is_file_encrypted(&file_path));
    }

    #[test]
    fn test_list_encrypted_files_patterns() {
        let _guard = TestIsolationGuard::new();
        let config = crate::config::Config {
            secrets: crate::config::SecretsConfig {
                encrypt_patterns: vec!["*.yaml".to_string(), "*.yml".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        // Create test files in the temp dir
        let test_files = [
            "secret.yaml",
            "config.enc.yaml",
            "data.yml",
            "ignore.txt",
            "backup.bak",
        ];
        for file_name in &test_files {
            let file_path = _guard.temp_dir().path().join(file_name);
            std::fs::write(&file_path, "test content").unwrap();
        }
        // Test the list_encrypted_files function
        let files = manager.list_encrypted_files().unwrap();
        // Should find secret.yaml, config.enc.yaml, and data.yml (3 files)
        // Should exclude ignore.txt (not yaml/yml) and backup.bak (excluded pattern)
        assert_eq!(
            files.len(),
            3,
            "Expected 3 files matching patterns, found {}",
            files.len()
        );
    }

    #[test]
    fn test_list_encrypted_files_encrypted() {
        let _guard = TestIsolationGuard::new();
        let config = crate::config::Config {
            secrets: crate::config::SecretsConfig {
                encrypt_patterns: vec!["*.enc.yaml".to_string()],
                ..Default::default()
            },
            ..Default::default()
        };
        let manager =
            SecretsManager::new(None, None, config, _guard.temp_dir().path().to_path_buf());
        // Create an encrypted file
        let file_path = _guard
            .temp_dir()
            .path()
            .join("test_list_encrypted_files_encrypted.enc.yaml");
        std::fs::write(&file_path, "dummy").unwrap();
        // Test the list_encrypted_files function
        let files = manager.list_encrypted_files().unwrap();
        // Should find the encrypted file
        assert!(files.iter().any(
            |(p, _)| p == std::path::Path::new("test_list_encrypted_files_encrypted.enc.yaml")
        ));
    }
}
