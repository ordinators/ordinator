use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

const STATE_FILE: &str = "readme_state.json";

#[derive(Serialize, Deserialize, Default)]
struct ReadmeState {
    config_hash: String,
    last_updated: u64,
}

fn get_state_file_path(dotfiles_dir: &Path) -> PathBuf {
    dotfiles_dir.join(STATE_FILE)
}

fn compute_config_hash(config: &crate::config::Config) -> String {
    use serde_json::json;
    let mut relevant = serde_json::Map::new();
    let update_on_changes = &config.readme.update_on_changes;
    for change_type in update_on_changes {
        match change_type.as_str() {
            "profiles" => {
                relevant.insert(
                    "profiles".to_string(),
                    serde_json::to_value(&config.profiles).unwrap_or(json!(null)),
                );
            }
            "bootstrap" => {
                // Collect all bootstrap_script fields
                let bootstraps: HashMap<_, _> = config
                    .profiles
                    .iter()
                    .filter_map(|(k, v)| v.bootstrap_script.as_ref().map(|s| (k, s)))
                    .collect();
                relevant.insert(
                    "bootstrap".to_string(),
                    serde_json::to_value(bootstraps).unwrap_or(json!(null)),
                );
            }
            "age_key" => {
                relevant.insert(
                    "age_key".to_string(),
                    serde_json::to_value(&config.secrets.age_key_file).unwrap_or(json!(null)),
                );
            }
            _ => {}
        }
    }
    let json = serde_json::to_vec(&relevant).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&json);
    format!("{:x}", hasher.finalize())
}

fn read_state(dotfiles_dir: &Path) -> Option<ReadmeState> {
    let path = get_state_file_path(dotfiles_dir);
    if let Ok(mut file) = File::open(path) {
        let mut buf = String::new();
        if file.read_to_string(&mut buf).is_ok() {
            if let Ok(state) = serde_json::from_str(&buf) {
                return Some(state);
            }
        }
    }
    None
}

fn write_state(dotfiles_dir: &Path, hash: &str) {
    let path = get_state_file_path(dotfiles_dir);
    let state = ReadmeState {
        config_hash: hash.to_string(),
        last_updated: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    if let Ok(json) = serde_json::to_string_pretty(&state) {
        let _ = fs::create_dir_all(path.parent().unwrap());
        let _ = fs::write(path, json);
    }
}

/// Configuration for README generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeConfig {
    #[serde(default)]
    pub auto_update: bool,

    #[serde(default)]
    pub update_on_changes: Vec<String>,
}

impl Default for ReadmeConfig {
    fn default() -> Self {
        Self {
            auto_update: false,
            update_on_changes: vec!["profiles".to_string(), "bootstrap".to_string()],
        }
    }
}

/// README manager for handling README operations
pub struct ReadmeManager {
    dry_run: bool,
}

impl ReadmeManager {
    /// Create a new README manager
    pub fn new(dry_run: bool) -> Self {
        let _ordinator_home = std::env::var("ORDINATOR_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".ordinator")
            });
        Self { dry_run }
    }

    /// Generate default README if none exists
    pub fn generate_default_readme(
        &self,
        _config: &crate::config::Config,
        dotfiles_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        let readme_path = dotfiles_dir.join("README.md");

        if readme_path.exists() {
            if !self.dry_run {
                eprintln!("README.md already exists");
            }
            return Ok(None);
        }

        if self.dry_run {
            eprintln!("DRY-RUN: Would generate README.md");
            return Ok(Some(readme_path));
        }

        // Get repository URL and branch
        let git_manager = crate::git::GitManager::new(dotfiles_dir.to_path_buf());
        let repo_url = git_manager.get_origin_url().unwrap_or(None);
        let branch = git_manager
            .get_default_branch()
            .unwrap_or_else(|_| "main".to_string());

        // Load config for config-aware README generation
        let (config, _) = crate::config::Config::load()?;
        let generator =
            READMEGenerator::new_with_repo_url_and_branch(false, false, repo_url, branch);
        let content = generator.generate_readme_with_config(&config)?;
        fs::write(&readme_path, content)?;

        Ok(Some(readme_path))
    }

    /// Interactive README customization
    pub fn interactive_customization(
        &self,
        _config: &crate::config::Config,
        dotfiles_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        use dialoguer::{Confirm, Editor, Input, MultiSelect};

        let readme_path = dotfiles_dir.join("README.md");

        if self.dry_run {
            eprintln!("DRY-RUN: Would run interactive README customization");
            return Ok(Some(readme_path));
        }

        // Prompt for project name and description
        let project_name: String = Input::new()
            .with_prompt("Project name for your dotfiles repo")
            .default("Dotfiles Repository".into())
            .interact_text()?;
        let project_desc: String = Input::new()
            .with_prompt("Short description")
            .default("Personal and work environment configuration managed by Ordinator.".into())
            .interact_text()?;

        // Prompt for profiles
        let mut profiles = Vec::new();
        loop {
            let profile: String = Input::new()
                .with_prompt("Add a profile (leave blank to finish)")
                .allow_empty(true)
                .interact_text()?;
            if profile.trim().is_empty() {
                break;
            }
            let desc: String = Input::new()
                .with_prompt(format!("Description for profile '{profile}'"))
                .default("No description".into())
                .interact_text()?;
            profiles.push((profile, desc));
        }
        if profiles.is_empty() {
            profiles.push((
                "work".to_string(),
                "Work environment configuration".to_string(),
            ));
            profiles.push((
                "personal".to_string(),
                "Personal environment configuration".to_string(),
            ));
        }

        // Prompt for sections to include
        let section_options = vec![
            "Quick Install",
            "Profiles",
            "AGE Key Setup",
            "Troubleshooting",
            "Security Notes",
        ];
        let selections = MultiSelect::new()
            .with_prompt("Select sections to include in your README")
            .items(&section_options)
            .defaults(&[true, true, true, true, true])
            .interact()?;

        // Build README content
        let mut content = String::new();
        content.push_str(&format!("# {project_name}\n\n{project_desc}\n\n"));
        for &idx in &selections {
            match section_options[idx] {
                "Quick Install" => {
                    let repo_url = "https://github.com/yourname/dotfiles.git";
                    let install_command = format!("curl -fsSL https://raw.githubusercontent.com/ordinators/ordinator/master/scripts/install.sh | sh && ordinator init {repo_url} && ordinator apply");
                    content.push_str(&format!(
                        "## Quick Install\n\n```bash\n{install_command}\n```\n\n"
                    ));
                }
                "Profiles" => {
                    content.push_str(
                        "## Profiles\n\nThis repository contains the following profiles:\n\n",
                    );
                    for (name, desc) in &profiles {
                        content.push_str(&format!("- **{name}**: {desc}\n"));
                    }
                    content.push_str("\nTo apply a profile:\n```bash\nordinator apply --profile <profile-name>\n```\n\n");
                    // After the profiles section, add Homebrew packages if present in config
                    if let Ok((config, _)) = crate::config::Config::load() {
                        let homebrew_section = READMEGenerator {
                            branch: "main".to_string(),
                        }
                        .generate_homebrew_packages_with_config(&config);
                        if !homebrew_section.is_empty() {
                            content.push_str(&homebrew_section);
                        }
                    }
                }
                "AGE Key Setup" => {
                    content.push_str("## AGE Key Setup\n\nThis repository uses encrypted secrets. You'll need to set up an AGE key:\n\n1. Generate an AGE key:\n```bash\nordinator secrets setup --profile <profile-name>\n```\n\n2. The key will be created at `~/.config/ordinator/age/<profile>.txt`\n\n3. **Never commit your AGE key to version control!**\n\n");
                }
                "Troubleshooting" => {
                    content.push_str("## Troubleshooting\n\n### Common Issues\n\n- **Broken symlinks**: Run `ordinator repair` to fix\n- **Missing files**: Run `ordinator apply` to recreate symlinks\n- **Secrets not decrypting**: Ensure your AGE key is in the correct location\n- **Permission errors**: Check file permissions and ownership\n\n");
                }
                "Security Notes" => {
                    content.push_str("## Security Notes\n\n- Keep your AGE key secure and never commit it to version control\n- Use different AGE keys for different environments (work/personal)\n- Regularly rotate your AGE keys\n- Be careful with Personal Access Tokens - they provide access to your repositories\n\n");
                }
                _ => {}
            }
        }

        // Preview and confirm
        println!("\n--- README Preview ---\n\n{content}\n---------------------\n");
        if !Confirm::new()
            .with_prompt("Save this README to README.md?")
            .default(true)
            .interact()?
        {
            eprintln!("Aborted by user. No README written.");
            return Ok(None);
        }

        // Optionally open in $EDITOR
        if Confirm::new()
            .with_prompt("Edit README in $EDITOR before saving?")
            .default(false)
            .interact()?
        {
            if let Some(edited) = Editor::new().edit(&content)? {
                fs::write(&readme_path, edited)?;
            } else {
                fs::write(&readme_path, content)?;
            }
        } else {
            fs::write(&readme_path, content)?;
        }

        println!("README.md written to {readme_path:?}");
        Ok(Some(readme_path))
    }

    /// Preview README content
    pub fn preview_readme(
        &self,
        config: &crate::config::Config,
        dotfiles_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        let readme_path = dotfiles_dir.join("README.md");

        if self.dry_run {
            eprintln!("DRY-RUN: Would preview README.md");
            return Ok(Some(readme_path));
        }

        // Get repository URL
        let git_manager = crate::git::GitManager::new(dotfiles_dir.to_path_buf());
        let repo_url = git_manager.get_origin_url().unwrap_or(None);

        let branch = "main".to_string(); // fallback for preview
        let generator =
            READMEGenerator::new_with_repo_url_and_branch(false, true, repo_url, branch);
        let content = generator.generate_readme_with_config(config)?;

        // Show preview
        println!("{content}");

        // TODO: Add interactive save prompt
        eprintln!("To save this README, run: ordinator readme");

        Ok(Some(readme_path))
    }

    /// Edit README in $EDITOR
    pub fn edit_readme(
        &self,
        config: &crate::config::Config,
        dotfiles_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        let readme_path = dotfiles_dir.join("README.md");

        if self.dry_run {
            eprintln!("DRY-RUN: Would edit README.md");
            return Ok(Some(readme_path));
        }

        // Generate README if it doesn't exist
        if !readme_path.exists() {
            // Get repository URL
            let git_manager = crate::git::GitManager::new(dotfiles_dir.to_path_buf());
            let repo_url = git_manager.get_origin_url().unwrap_or(None);

            let branch = "main".to_string(); // fallback for edit
            let generator =
                READMEGenerator::new_with_repo_url_and_branch(false, false, repo_url, branch);
            let content = generator.generate_readme_with_config(config)?;
            fs::write(&readme_path, content)?;
            eprintln!("Generated README.md for editing");
        }

        // Get $EDITOR
        let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());

        // Open in editor
        let status = Command::new(&editor).arg(&readme_path).status()?;

        if !status.success() {
            return Err(anyhow::anyhow!("Editor exited with status: {}", status));
        }

        Ok(Some(readme_path))
    }
}

/// README generator with customization options
pub struct READMEGenerator {
    branch: String,
}

impl READMEGenerator {
    /// Create a new README generator with branch
    pub fn new_with_repo_url_and_branch(
        _interactive: bool,
        _preview: bool,
        _repo_url: Option<String>,
        branch: String,
    ) -> Self {
        Self { branch }
    }

    /// Generate README content from template with config
    pub fn generate_readme_with_config(&self, config: &crate::config::Config) -> Result<String> {
        let mut content = String::new();

        // Add header
        content.push_str(&self.generate_header());

        // Add sections
        content.push_str(&self.generate_quick_install());
        content.push_str(&self.generate_profiles_with_config(config));
        content.push_str(&self.generate_homebrew_packages_with_config(config));
        content.push_str(&self.generate_age_key());
        content.push_str(&self.generate_troubleshooting());
        content.push_str(&self.generate_security());

        // Add footer
        content.push_str(&self.generate_footer());

        Ok(content)
    }

    /// Generate Homebrew packages section for all profiles
    fn generate_homebrew_packages_with_config(&self, config: &crate::config::Config) -> String {
        let mut content = String::new();
        let mut any_packages = false;

        for (profile_name, profile_config) in &config.profiles {
            let mut formulas = profile_config.homebrew_formulas.clone();
            let mut casks = profile_config.homebrew_casks.clone();
            if formulas.is_empty() && casks.is_empty() {
                continue;
            }
            any_packages = true;
            formulas.sort();
            casks.sort();
            let emoji = match profile_name.as_str() {
                "work" => "\u{1F4BC}",           // 💼
                "personal" => "\u{1F3E0}",       // 🏠
                "laptop" => "\u{1F4BB}",         // 💻
                "default" => "\u{2699}\u{FE0F}", // ⚙️
                _ => "\u{2699}\u{FE0F}",         // ⚙️
            };
            content.push_str(&format!(
                "<details>\n  <summary><strong>{emoji} {profile_name} Profile Packages</strong></summary>\n  <div style=\"margin-top:10px; padding:10px; border:1px solid #ddd; border-radius:8px;\">\n"
            ));
            if !formulas.is_empty() {
                content.push_str("    <p><strong>Formulas:</strong> ");
                for (i, formula) in formulas.iter().enumerate() {
                    let url = format!("https://formulae.brew.sh/formula/{formula}");
                    content.push_str(&format!(
                        "<a href=\"{url}\" target=\"_blank\">{formula}</a>"
                    ));
                    if i < formulas.len() - 1 {
                        content.push_str(" • ");
                    }
                }
                content.push_str("</p>\n");
            }
            if !casks.is_empty() {
                content.push_str("    <p><strong>Casks:</strong> ");
                for (i, cask) in casks.iter().enumerate() {
                    let url = format!("https://formulae.brew.sh/cask/{cask}");
                    content.push_str(&format!("<a href=\"{url}\" target=\"_blank\">{cask}</a>"));
                    if i < casks.len() - 1 {
                        content.push_str(" • ");
                    }
                }
                content.push_str("</p>\n");
            }
            content.push_str("  </div>\n</details>\n\n");
        }
        if any_packages {
            format!("## Homebrew Packages\n\n{content}")
        } else {
            String::new()
        }
    }

    // Template generation methods
    fn generate_header(&self) -> String {
        String::from("# Dotfiles Repository\n\n")
    }

    fn generate_quick_install(&self) -> String {
        let branch = &self.branch;
        let replicate_oneliner = format!(
            "bash <(curl -fsSL https://raw.githubusercontent.com/{{username}}/{{repo}}/{branch}/replicate.sh)"
        );
        let note = "If your repository uses a different default branch (e.g., master), update the one-liner to match your branch name.";
        format!("## Quick Install\n\n```bash\n{replicate_oneliner}\n```\n\n> {note}\n\n")
    }

    fn generate_profiles_with_config(&self, config: &crate::config::Config) -> String {
        let mut content =
            String::from("## Profiles\n\nThis repository contains the following profiles:\n\n");

        for (profile_name, profile_config) in &config.profiles {
            let description = profile_config
                .description
                .as_deref()
                .unwrap_or("No description");
            content.push_str(&format!("- **{profile_name}**: {description}\n"));
        }

        content.push_str(
            "\nTo apply a profile:\n```bash\nordinator apply --profile <profile-name>\n```\n\n",
        );

        content
    }

    fn generate_age_key(&self) -> String {
        String::from("## AGE Key Setup\n\nThis repository uses encrypted secrets. You'll need to set up an AGE key:\n\n1. Generate an AGE key:\n```bash\nordinator secrets setup --profile <profile-name>\n```\n\n2. The key will be created at `~/.config/ordinator/age/<profile>.txt`\n\n3. **Never commit your AGE key to version control!**\n\n")
    }

    fn generate_troubleshooting(&self) -> String {
        String::from("## Troubleshooting\n\n### Common Issues\n\n- **Broken symlinks**: Run `ordinator repair` to fix\n- **Missing files**: Run `ordinator apply` to recreate symlinks\n- **Secrets not decrypting**: Ensure your AGE key is in the correct location\n- **Permission errors**: Check file permissions and ownership\n\n")
    }

    fn generate_security(&self) -> String {
        String::from("## Security Notes\n\n- Keep your AGE key secure and never commit it to version control\n- Use different AGE keys for different environments (work/personal)\n- Regularly rotate your AGE keys\n- Be careful with Personal Access Tokens - they provide access to your repositories\n\n")
    }

    fn generate_footer(&self) -> String {
        String::from("---\n\nGenerated by [Ordinator](https://github.com/ordinators/ordinator) - Dotfiles and Environment Manager for macOS\n")
    }
}

/// Check if README needs updating based on config changes
/// Uses hash comparison to detect if relevant config sections have changed
pub fn readme_needs_update(config: &crate::config::Config, dotfiles_dir: &Path) -> bool {
    let current_hash = compute_config_hash(config);
    let state = read_state(dotfiles_dir);
    state
        .as_ref()
        .map(|s| s.config_hash != current_hash)
        .unwrap_or(true)
}

/// Auto-update README
pub fn auto_update_readme(config: &crate::config::Config, dotfiles_dir: &Path) -> Result<()> {
    let readme_manager = ReadmeManager::new(false); // Not dry-run for auto-update
    if let Some(readme_path) = readme_manager.generate_default_readme(config, dotfiles_dir)? {
        eprintln!("📝 Auto-updated README.md: {}", readme_path.display());
    } else {
        eprintln!("📝 README.md is up to date.");
    }

    // Update state hash
    let current_hash = compute_config_hash(config);
    write_state(dotfiles_dir, &current_hash);

    Ok(())
}
