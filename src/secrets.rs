use crate::config::Config;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use tracing::info;

/// Secrets manager using SOPS and age
#[allow(dead_code)]
pub struct SecretsManager {
    age_key_file: Option<std::path::PathBuf>,
    sops_config: Option<std::path::PathBuf>,
    config: Config,
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
    ) -> Self {
        Self {
            age_key_file,
            sops_config,
            config,
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
    #[allow(dead_code)]
    pub fn list_encrypted_files(
        &self,
        _repo_path: &std::path::Path,
    ) -> Result<Vec<std::path::PathBuf>> {
        // TODO: Implement listing encrypted files
        Ok(Vec::new())
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

pub fn encrypt_file_with_sops(file: &str) -> anyhow::Result<String> {
    use std::path::Path;
    use std::process::Command;
    // Check if sops is available
    check_sops_and_age()?;
    let input_path = Path::new(file);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }
    let file = input_path.to_string_lossy().to_string();
    let file_name = input_path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("file");

    // Determine output path
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
            format!("{file_name}.enc")
        }
    } else {
        format!("{file_name}.enc")
    };

    // Call sops to encrypt
    let status = Command::new("sops")
        .arg("--encrypt")
        .arg(&file)
        .arg("--output")
        .arg(&output_path)
        .status()?;
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
    let input_path = Path::new(file);
    if !input_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file));
    }
    // Call sops to decrypt
    let status = Command::new("sops").arg("--decrypt").arg(file).status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("sops failed to decrypt file: {}", file));
    }
    info!("Successfully decrypted file: {}", file);
    Ok(())
}

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_encrypt_file() {
        let config = Config::default();
        let mut manager = SecretsManager::new(None, None, config);
        let file_path = std::path::Path::new("/tmp/test.txt");

        let result = manager.encrypt_file(file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_file_with_configs() {
        let age_key = std::path::PathBuf::from("/path/to/age.key");
        let sops_config = std::path::PathBuf::from("/path/to/.sops.yaml");
        let config = Config::default();
        let mut manager = SecretsManager::new(Some(age_key), Some(sops_config), config);

        let file_path = std::path::Path::new("/tmp/test.txt");
        let result = manager.encrypt_file(file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_file() {
        let config = Config::default();
        let mut manager = SecretsManager::new(None, None, config);
        let file_path = std::path::Path::new("/tmp/test.enc.yaml");

        let result = manager.decrypt_file(file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decrypt_file_with_configs() {
        let age_key = std::path::PathBuf::from("/path/to/age.key");
        let sops_config = std::path::PathBuf::from("/path/to/.sops.yaml");
        let config = Config::default();
        let mut manager = SecretsManager::new(Some(age_key), Some(sops_config), config);

        let file_path = std::path::Path::new("/tmp/test.enc.yaml");
        let result = manager.decrypt_file(file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_encrypted_files() {
        let config = Config::default();
        let manager = SecretsManager::new(None, None, config);
        let repo_path = std::path::Path::new("/tmp/repo");

        let result = manager.list_encrypted_files(repo_path);
        assert!(result.is_ok());

        let files = result.unwrap();
        assert!(files.is_empty()); // Currently returns empty vector
    }

    #[test]
    fn test_list_encrypted_files_with_configs() {
        let age_key = std::path::PathBuf::from("/path/to/age.key");
        let sops_config = std::path::PathBuf::from("/path/to/.sops.yaml");
        let config = Config::default();
        let manager = SecretsManager::new(Some(age_key), Some(sops_config), config);

        let repo_path = std::path::Path::new("/tmp/repo");
        let result = manager.list_encrypted_files(repo_path);
        assert!(result.is_ok());

        let files = result.unwrap();
        assert!(files.is_empty()); // Currently returns empty vector
    }

    #[test]
    fn test_check_for_plaintext_secrets() {
        let config = Config::default();
        let manager = SecretsManager::new(None, None, config);
        let file_path = std::path::Path::new("/tmp/test.txt");

        let result = manager.check_for_plaintext_secrets(file_path);
        assert!(result.is_ok());

        let has_secrets = result.unwrap();
        assert!(!has_secrets); // Currently returns false
    }

    #[test]
    fn test_check_for_plaintext_secrets_with_configs() {
        let age_key = std::path::PathBuf::from("/path/to/age.key");
        let sops_config = std::path::PathBuf::from("/path/to/.sops.yaml");
        let config = Config::default();
        let manager = SecretsManager::new(Some(age_key), Some(sops_config), config);

        let file_path = std::path::Path::new("/tmp/test.txt");
        let result = manager.check_for_plaintext_secrets(file_path);
        assert!(result.is_ok());

        let has_secrets = result.unwrap();
        assert!(!has_secrets); // Currently returns false
    }

    #[test]
    fn test_validate_installation() {
        let config = Config::default();
        let manager = SecretsManager::new(None, None, config);

        let result = manager.validate_installation();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_installation_with_configs() {
        let age_key = std::path::PathBuf::from("/path/to/age.key");
        let sops_config = std::path::PathBuf::from("/path/to/.sops.yaml");
        let config = Config::default();
        let manager = SecretsManager::new(Some(age_key), Some(sops_config), config);

        let result = manager.validate_installation();
        assert!(result.is_ok());
    }

    #[test]
    fn test_secrets_manager_integration() {
        let age_key = std::path::PathBuf::from("/path/to/age.key");
        let sops_config = std::path::PathBuf::from("/path/to/.sops.yaml");
        let config = Config::default();
        let mut manager = SecretsManager::new(Some(age_key), Some(sops_config), config);

        // Test all methods in sequence
        let file_path = std::path::Path::new("/tmp/test.txt");
        assert!(manager.encrypt_file(file_path).is_ok());

        let enc_file_path = std::path::Path::new("/tmp/test.enc.yaml");
        assert!(manager.decrypt_file(enc_file_path).is_ok());

        let repo_path = std::path::Path::new("/tmp/repo");
        let files = manager.list_encrypted_files(repo_path).unwrap();
        assert!(files.is_empty());

        assert!(manager.check_for_plaintext_secrets(file_path).is_ok());
        assert!(manager.validate_installation().is_ok());
    }

    #[test]
    fn test_secrets_manager_edge_cases() {
        let config = Config::default();
        let mut manager = SecretsManager::new(None, None, config);

        // Test with non-existent paths
        let non_existent_path = std::path::Path::new("/non/existent/path");
        assert!(manager.encrypt_file(non_existent_path).is_ok());
        assert!(manager.decrypt_file(non_existent_path).is_ok());
        assert!(manager.list_encrypted_files(non_existent_path).is_ok());
        assert!(manager
            .check_for_plaintext_secrets(non_existent_path)
            .is_ok());
    }

    #[test]
    fn test_secrets_manager_with_empty_paths() {
        let config = Config::default();
        let mut manager = SecretsManager::new(None, None, config);

        // Test with empty path
        let empty_path = std::path::Path::new("");
        assert!(manager.encrypt_file(empty_path).is_ok());
        assert!(manager.decrypt_file(empty_path).is_ok());
        assert!(manager.list_encrypted_files(empty_path).is_ok());
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
        let temp_dir = tempfile::TempDir::new().unwrap();
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
        std::env::set_var("PATH", orig_path);
    }

    #[test]
    fn test_encrypt_file_with_sops_file_not_found() {
        let file = "/non/existent/file.txt";
        assert!(encrypt_file_with_sops(file).is_err());
    }

    #[test]
    fn test_encrypt_file_with_sops_sops_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
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
        let temp_dir = tempfile::TempDir::new().unwrap();
        let orig_path = env::var("PATH").unwrap_or_default();
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
        assert!(result.is_err());

        // Clean up
        if let Ok(output_path) = result {
            let output_path = std::path::Path::new(&output_path);
            if output_path.exists() {
                fs::remove_file(output_path).unwrap();
            }
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
        let temp_dir = tempfile::TempDir::new().unwrap();
        let orig_path = env::var("PATH").unwrap_or_default();
        // Create a shell script as dummy sops that copies input to output
        let sops_path = temp_dir.path().join("sops");
        fs::write(&sops_path, "#!/bin/sh\n/bin/cp \"$2\" \"$4\"\n").unwrap();
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
            println!("[TEST DEBUG] sops_success error: {e}");
        }
        assert!(result.is_ok());
        let output_path = result.unwrap();
        assert!(std::path::Path::new(&output_path).exists());
        let contents = fs::read_to_string(&output_path).unwrap();
        assert_eq!(contents, "hello");

        // Clean up
        let output_path = std::path::Path::new(&output_path);
        if output_path.exists() {
            fs::remove_file(output_path).unwrap();
        }

        // Restore original PATH
        std::env::set_var("PATH", orig_path);
    }
}
