use anyhow::{anyhow, Result};
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};
use url::Url;

use crate::config::Config;
use crate::git::GitManager;

/// Repository manager for handling remote repository initialization
pub struct RepoManager {
    target_dir: PathBuf,
}

impl RepoManager {
    /// Create a new repository manager
    pub fn new(target_dir: PathBuf) -> Self {
        Self { target_dir }
    }

    /// Initialize from a remote repository URL
    pub async fn init_from_url(&self, repo_url: &str, force: bool) -> Result<()> {
        info!("Initializing from repository URL: {}", repo_url);

        // Parse the repository URL
        let repo_info = self.parse_github_url(repo_url)?;
        info!("Repository info: {:?}", repo_info);

        // Check if target directory exists
        if self.target_dir.exists() {
            if !force {
                return Err(anyhow!(
                    "Target directory '{}' already exists. Use --force to overwrite.",
                    self.target_dir.display()
                ));
            }
            // TODO: Implement safe directory removal
            warn!("Target directory exists, would remove with --force");
        }

        // Try Git clone first (for public repositories)
        if let Ok(()) = self.try_git_clone(repo_url).await {
            info!("Successfully cloned repository using Git");
            return self.validate_and_setup_repository();
        }

        // Fall back to source archive download (for private repositories)
        info!("Git clone failed, trying source archive download");
        if let Ok(()) = self.download_source_archive(&repo_info).await {
            info!("Successfully downloaded repository archive");
            return self.validate_and_setup_repository();
        }

        // If both methods fail, guide user to manual setup
        Err(anyhow!(
            "Failed to initialize from repository. Please ensure:\n\
             - The repository is accessible (public or you have access)\n\
             - Your SSH key is properly configured (for SSH URLs)\n\
             - You have network connectivity\n\
             \n\
             You can manually clone the repository and run 'ordinator init' in the directory."
        ))
    }

    /// Parse GitHub URL to extract repository information
    pub fn parse_github_url(&self, url: &str) -> Result<GitHubRepoInfo> {
        // Try to parse as a standard URL first
        if let Ok(parsed_url) = Url::parse(url) {
            // Only accept GitHub domains
            let host = parsed_url.host_str().unwrap_or("");
            if !host.ends_with("github.com") {
                return Err(anyhow!("Invalid GitHub URL format"));
            }

            // Reject if URL has fragment or query
            if parsed_url.fragment().is_some() || parsed_url.query().is_some() {
                return Err(anyhow!("Invalid GitHub URL format"));
            }

            let raw_segments: Vec<&str> = parsed_url.path().split('/').collect();
            if raw_segments.len() < 3 {
                // e.g., /user/repo or /user/repo.git
                return Err(anyhow!("Invalid GitHub URL format"));
            }
            // Ignore the first segment (always empty due to leading slash)
            if raw_segments[1..].iter().any(|s| s.is_empty()) {
                // Reject any empty segment after the first (consecutive slashes)
                return Err(anyhow!("Invalid GitHub URL format"));
            }
            let path_segments: Vec<&str> = raw_segments
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect();
            if path_segments.len() != 2 {
                // Must be exactly two segments: owner/repo
                return Err(anyhow!("Invalid GitHub URL format"));
            }
            let owner = path_segments[0];
            let repo_name = path_segments[1].trim_end_matches(".git");

            // Reject empty owner or repo names
            if owner.trim().is_empty() || repo_name.trim().is_empty() {
                return Err(anyhow!("Invalid GitHub URL format"));
            }

            // Only allow valid GitHub characters (alphanumeric, hyphens, underscores)
            let valid = |s: &str| {
                s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            };
            if !valid(owner) || !valid(repo_name) {
                return Err(anyhow!("Invalid GitHub URL format"));
            }

            let is_ssh = parsed_url.scheme() == "ssh" || parsed_url.scheme() == "git";
            return Ok(GitHubRepoInfo {
                owner: owner.to_string(),
                repo: repo_name.to_string(),
                is_ssh,
                full_url: url.to_string(),
            });
        }

        // Fallback: handle scp-like SSH URLs (git@github.com:user/repo.git)
        // Example: git@github.com:user/repo.git
        if let Some((user_host, path)) = url.split_once(':') {
            // Only accept GitHub SSH URLs
            if user_host.ends_with("github.com") {
                // Reject if path contains fragment or query
                if path.contains('#') || path.contains('?') {
                    return Err(anyhow!("Invalid GitHub URL format"));
                }
                let path = path.trim_start_matches('/');
                let mut segments = path.split('/');
                let owner = segments
                    .next()
                    .ok_or_else(|| anyhow!("Invalid SSH GitHub URL: missing owner"))?;
                let repo_name = segments
                    .next()
                    .ok_or_else(|| anyhow!("Invalid SSH GitHub URL: missing repo"))?
                    .trim_end_matches(".git");
                // Reject if owner or repo is empty or if there are extra segments
                if owner.trim().is_empty()
                    || repo_name.trim().is_empty()
                    || segments.next().is_some()
                {
                    return Err(anyhow!("Invalid GitHub URL format"));
                }
                // Only allow valid GitHub characters (alphanumeric, hyphens, underscores)
                let valid = |s: &str| {
                    s.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                };
                if !valid(owner) || !valid(repo_name) {
                    return Err(anyhow!("Invalid GitHub URL format"));
                }
                return Ok(GitHubRepoInfo {
                    owner: owner.to_string(),
                    repo: repo_name.to_string(),
                    is_ssh: true,
                    full_url: url.to_string(),
                });
            }
        }
        Err(anyhow!("Invalid GitHub URL format"))
    }

    /// Try to clone repository using Git
    async fn try_git_clone(&self, repo_url: &str) -> Result<()> {
        info!("Attempting Git clone: {}", repo_url);

        // Check if we're in test mode
        if std::env::var("ORDINATOR_TEST_MODE").ok().as_deref() == Some("1") {
            info!("[TEST MODE] Simulating Git clone for: {}", repo_url);

            // Create the target directory structure for testing
            std::fs::create_dir_all(&self.target_dir)?;

            // Simulate different failure scenarios based on the URL
            if repo_url.contains("nonexistent") || repo_url.contains("private") {
                return Err(anyhow!("Git clone failed: Repository not found"));
            }

            // For valid-looking URLs, simulate success but create a minimal structure
            // Create a .git directory to make it look like a git repo
            let git_dir = self.target_dir.join(".git");
            std::fs::create_dir_all(&git_dir)?;

            info!("[TEST MODE] Git clone simulation successful");
            return Ok(());
        }

        let output = Command::new("git")
            .arg("clone")
            .arg(repo_url)
            .arg(&self.target_dir)
            .output()?;

        if output.status.success() {
            info!("Git clone successful");
            Ok(())
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("Git clone failed: {}", error);
            Err(anyhow!("Git clone failed: {}", error))
        }
    }

    /// Download repository as source archive
    async fn download_source_archive(&self, repo_info: &GitHubRepoInfo) -> Result<()> {
        info!(
            "Downloading source archive for {}/{}",
            repo_info.owner, repo_info.repo
        );

        // Check if we're in test mode
        if std::env::var("ORDINATOR_TEST_MODE").ok().as_deref() == Some("1") {
            info!(
                "[TEST MODE] Simulating source archive download for {}/{}",
                repo_info.owner, repo_info.repo
            );

            // Create the target directory structure for testing
            std::fs::create_dir_all(&self.target_dir)?;

            // Simulate different failure scenarios based on the repo info
            if repo_info.owner.contains("nonexistent") || repo_info.owner.contains("private") {
                return Err(anyhow!("Failed to download archive: HTTP 404"));
            }

            // For valid-looking repos, simulate success but create a minimal structure
            // Create a .git directory to make it look like a git repo
            let git_dir = self.target_dir.join(".git");
            std::fs::create_dir_all(&git_dir)?;

            info!("[TEST MODE] Source archive download simulation successful");
            return Ok(());
        }

        // Construct archive URL
        let archive_url = format!(
            "https://github.com/{}/{}/archive/refs/heads/master.tar.gz",
            repo_info.owner, repo_info.repo
        );

        info!("Archive URL: {}", archive_url);

        // Download the archive
        let response = reqwest::get(&archive_url).await?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "Failed to download archive: HTTP {}",
                response.status()
            ));
        }

        // Create target directory
        std::fs::create_dir_all(&self.target_dir)?;

        // Download and extract the archive
        let bytes = response.bytes().await?;
        let mut archive = flate2::read::GzDecoder::new(&bytes[..]);
        let mut tar = tar::Archive::new(&mut archive);

        tar.unpack(&self.target_dir)?;

        // Rename the extracted directory to match the target
        let extracted_dir = self.target_dir.join(format!("{}-master", repo_info.repo));
        if extracted_dir.exists() {
            // Move contents up one level
            for entry in std::fs::read_dir(&extracted_dir)? {
                let entry = entry?;
                let dest_path = self.target_dir.join(entry.file_name());
                std::fs::rename(entry.path(), dest_path)?;
            }
            std::fs::remove_dir(extracted_dir)?;
        }

        info!("Archive downloaded and extracted successfully");
        Ok(())
    }

    /// Validate repository structure and set up configuration
    fn validate_and_setup_repository(&self) -> Result<()> {
        info!("Validating repository structure");

        // Check for ordinator.toml
        let config_path = self.target_dir.join("ordinator.toml");
        if !config_path.exists() {
            return Err(anyhow!(
                "Repository does not contain 'ordinator.toml'. \
                 This may not be a valid Ordinator repository."
            ));
        }

        // Load and validate configuration
        let _config = Config::from_file(&config_path)?;
        info!("Configuration loaded successfully");

        // Initialize Git repository if not already present
        let git_manager = GitManager::new(self.target_dir.clone());
        if !git_manager.exists() {
            git_manager.init()?;
            info!("Git repository initialized");
        }

        info!("Repository initialization completed successfully");
        Ok(())
    }
}

/// Information about a GitHub repository
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitHubRepoInfo {
    owner: String,
    repo: String,
    is_ssh: bool,
    full_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_parse_github_url_https() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());

        // Only accept strict URLs
        let valid_urls = vec![
            "https://github.com/user/repo",
            "https://github.com/user/repo.git",
        ];
        for url in valid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_ok(), "Should succeed for valid URL: {url}");
            let info = result.unwrap();
            assert_eq!(info.owner, "user");
            assert_eq!(info.repo, "repo");
            assert!(!info.is_ssh);
        }

        // Should reject trailing slashes, fragments, query params, extra segments, etc.
        let invalid_urls = vec![
            "https://github.com/user/repo/",
            "https://github.com/user/repo///",
            "https://github.com/user/repo.git/",
            "https://github.com/user/repo.git#main",
            "https://github.com/user/repo.git?ref=main",
            "https://github.com/user/repo/extra",
        ];
        for url in invalid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_err(), "Should fail for invalid URL: {url}");
        }
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());

        let valid_urls = vec!["git@github.com:user/repo", "git@github.com:user/repo.git"];
        for url in valid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_ok(), "Should succeed for valid SSH URL: {url}");
            let info = result.unwrap();
            assert_eq!(info.owner, "user");
            assert_eq!(info.repo, "repo");
            assert!(info.is_ssh);
        }

        // Should reject trailing slashes, fragments, query params, etc.
        let invalid_urls = vec![
            "git@github.com:user/repo/",
            "git@github.com:user/repo.git/",
            "git@github.com:user/repo.git#main",
            "git@github.com:user/repo.git?ref=main",
            "git@github.com:user/repo/extra",
        ];
        for url in invalid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_err(), "Should fail for invalid SSH URL: {url}");
        }
    }

    #[test]
    fn test_parse_github_url_case_sensitivity() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Accept case-sensitive owner/repo
        let url = "https://github.com/USER/REPO.git";
        let result = manager.parse_github_url(url);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.owner, "USER");
        assert_eq!(info.repo, "REPO");
    }

    #[test]
    fn test_parse_github_url_edge_cases() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Only strict valid URLs should succeed
        let valid_urls = vec![
            ("https://github.com/user/repo", "user", "repo"),
            ("https://github.com/user/repo.git", "user", "repo"),
            ("git@github.com:user/repo", "user", "repo"),
            ("git@github.com:user/repo.git", "user", "repo"),
        ];
        for (url, expected_owner, expected_repo) in valid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_ok(), "Should succeed for valid URL: {url}");
            let info = result.unwrap();
            assert_eq!(info.owner, expected_owner);
            assert_eq!(info.repo, expected_repo);
        }
        // All other edge cases should fail
        let invalid_urls = vec![
            "https://github.com/user/repo/",
            "git@github.com:user/repo/",
            "https://github.com/user/repo.git/",
            "git@github.com:user/repo.git/",
            "https://github.com/user/repo.git#main",
            "git@github.com:user/repo.git#main",
            "https://github.com/user/repo.git?ref=main",
            "git@github.com:user/repo.git?ref=main",
            "https://github.com/user/repo/extra",
            "git@github.com:user/repo/extra",
        ];
        for url in invalid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_err(), "Should fail for edge case URL: {url}");
        }
    }

    #[test]
    fn test_parse_github_url_special_characters() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Accept hyphens and underscores
        let valid_urls = vec![
            "https://github.com/user-name/repo_name",
            "https://github.com/user-name/repo_name.git",
            "git@github.com:user-name/repo_name",
            "git@github.com:user-name/repo_name.git",
        ];
        for url in valid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_ok(), "Should handle hyphens/underscores: {url}");
        }
        // Reject spaces and extra segments
        let invalid_urls = vec![
            "https://github.com/user name/repo.git",
            "git@github.com:user name/repo.git",
            "https://github.com/user/repo name.git",
            "https://github.com/user/repo/extra",
        ];
        for url in invalid_urls {
            let result = manager.parse_github_url(url);
            assert!(
                result.is_err(),
                "Should fail for URL with spaces or extra segments: {url}"
            );
        }
    }

    #[test]
    fn test_parse_github_url_very_long() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Accept very long but valid URLs
        let long_owner = "a".repeat(100);
        let long_repo = "b".repeat(100);
        let long_url = format!("https://github.com/{long_owner}/{long_repo}.git");
        let result = manager.parse_github_url(&long_url);
        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.owner, long_owner);
        assert_eq!(info.repo, long_repo);
        // Reject with trailing slash
        let long_url_slash = format!("https://github.com/{long_owner}/{long_repo}/");
        let result = manager.parse_github_url(&long_url_slash);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_url_with_fragments() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Should reject fragments
        let result = manager.parse_github_url("https://github.com/user/repo.git#main");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_url_with_query_params() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Should reject query params
        let result = manager.parse_github_url("https://github.com/user/repo.git?ref=main");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_url_with_trailing_slashes() {
        let temp_dir = tempdir().unwrap();
        let manager = RepoManager::new(temp_dir.path().to_path_buf());
        // Should reject trailing slashes
        let valid_url = "https://github.com/user/repo";
        let invalid_urls = vec![
            "https://github.com/user/repo/",
            "https://github.com/user/repo///",
        ];
        let result = manager.parse_github_url(valid_url);
        assert!(result.is_ok());
        for url in invalid_urls {
            let result = manager.parse_github_url(url);
            assert!(result.is_err(), "Should fail for trailing slash: {url}");
        }
    }
}
