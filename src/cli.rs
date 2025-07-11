use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

use crate::config::Config;
use crate::git::GitManager;

#[derive(Parser)]
#[command(name = "ordinator")]
#[command(about = "Dotfiles and Environment Manager for macOS")]
#[command(version)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable dry-run mode (simulate operations without making changes)
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress status messages (only show errors)
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new dotfiles repository
    Init {
        /// Repository URL to clone from (GitHub HTTPS or SSH)
        #[arg(value_name = "REPO_URL")]
        repo_url: Option<String>,

        /// Target directory for the repository (defaults to current directory)
        #[arg(value_name = "TARGET_DIR")]
        target_dir: Option<String>,

        /// Profile to use for initialization (when not cloning from repo)
        #[arg(long, default_value = "default")]
        profile: String,

        /// Force overwrite existing directory
        #[arg(long)]
        force: bool,
    },

    /// Add a file to the dotfiles repository
    Add {
        /// File or directory to add
        #[arg(required = true)]
        path: String,

        /// Profile to associate with this file
        #[arg(long)]
        profile: Option<String>,
    },

    /// Remove a file from the dotfiles repository
    Remove {
        /// File or directory to remove
        #[arg(required = true)]
        path: String,
    },

    /// Commit changes to the repository
    Commit {
        /// Commit message
        #[arg(short, long, required = true)]
        message: String,

        /// Skip secrets scanning and commit anyway
        #[arg(long)]
        force: bool,
    },

    /// Push changes to remote repository
    Push {
        /// Repository URL to push to (sets remote if not configured)
        #[arg(value_name = "REPO_URL")]
        repo_url: Option<String>,

        /// Force push (use with caution)
        #[arg(long)]
        force: bool,
    },

    /// Pull changes from remote repository
    Pull {
        /// Rebase on pull
        #[arg(long)]
        rebase: bool,
    },

    /// Sync with remote repository (pull then push)
    Sync {
        /// Force push after sync
        #[arg(long)]
        force: bool,
    },

    /// Show repository status
    Status {
        /// Show detailed status
        #[arg(long)]
        verbose: bool,
    },

    /// Apply dotfiles to the current system
    Apply {
        /// Profile to apply
        #[arg(long, default_value = "default")]
        profile: String,

        /// Skip bootstrap script execution
        #[arg(long)]
        skip_bootstrap: bool,

        /// Skip secrets decryption
        #[arg(long)]
        skip_secrets: bool,

        /// Skip Homebrew package installation
        #[arg(long)]
        skip_brew: bool,

        /// Force overwrite existing files (use with caution)
        #[arg(long)]
        force: bool,
    },

    /// Repair broken symlinks
    Repair {
        /// Profile to repair (defaults to all profiles)
        #[arg(long)]
        profile: Option<String>,

        /// Show detailed repair information
        #[arg(long)]
        verbose: bool,
    },

    /// List available profiles
    Profiles {
        /// Show detailed profile information
        #[arg(long)]
        verbose: bool,
    },

    /// Manage secrets
    Secrets {
        #[command(subcommand)]
        subcommand: SecretCommands,
    },

    /// Generate system script for manual execution
    GenerateScript {
        /// Output file path
        #[arg(short, long, default_value = "ordinator-system.sh")]
        output: String,

        /// Profile to use
        #[arg(long, default_value = "default")]
        profile: String,
    },

    /// Execute bootstrap script for a profile
    Bootstrap {
        /// Profile to bootstrap
        #[arg(long, default_value = "default")]
        profile: String,

        /// Open the bootstrap script in $EDITOR for editing
        #[arg(long)]
        edit: bool,
    },

    /// Manage Homebrew packages
    Brew {
        #[command(subcommand)]
        subcommand: BrewCommands,
    },

    /// Manage README generation
    Readme {
        #[command(subcommand)]
        subcommand: ReadmeCommands,
    },
}

#[derive(Subcommand)]
pub enum SecretCommands {
    /// Encrypt a file with SOPS
    Encrypt {
        /// File to encrypt
        #[arg(required = true)]
        file: String,
    },

    /// Decrypt a file with SOPS
    Decrypt {
        /// File to decrypt
        #[arg(required = true)]
        file: String,
    },

    /// List encrypted files
    List {
        /// Show file paths only
        #[arg(long)]
        paths_only: bool,
    },

    /// Check for SOPS and age installation
    Check,

    /// Set up SOPS and age for secrets management
    Setup {
        /// Profile to set up (defaults to 'default')
        #[arg(long, default_value = "default")]
        profile: String,

        /// Force overwrite existing configuration
        #[arg(long)]
        force: bool,
    },

    /// Scan for plaintext secrets in tracked files
    Scan {
        /// Profile to scan (defaults to all profiles)
        #[arg(long)]
        profile: Option<String>,

        /// Show detailed information about found secrets
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
pub enum BrewCommands {
    /// Export current Homebrew packages to config
    Export {
        /// Profile to export packages for
        #[arg(long, default_value = "default")]
        profile: String,

        /// Include version information
        #[arg(long)]
        with_versions: bool,

        /// Force overwrite existing package list
        #[arg(long)]
        force: bool,
    },

    /// Install Homebrew packages for a profile
    Install {
        /// Profile to install packages for
        #[arg(long, default_value = "default")]
        profile: String,

        /// Skip interactive prompts
        #[arg(long)]
        non_interactive: bool,

        /// Force installation without confirmation
        #[arg(long)]
        force: bool,
    },

    /// List Homebrew packages for a profile
    List {
        /// Profile to list packages for
        #[arg(long, default_value = "default")]
        profile: String,

        /// Show detailed package information
        #[arg(long)]
        verbose: bool,
    },
}

#[derive(Subcommand)]
pub enum ReadmeCommands {
    /// Generate default README (if none exists)
    Default,

    /// Interactive README customization
    Interactive,

    /// Preview generated README before saving
    Preview,

    /// Edit existing README in $EDITOR
    Edit,
}

fn check_file_conflicts(config: &Config, file_path: &str, target_profile: &str) -> Vec<String> {
    let mut conflicts = Vec::new();
    for (profile_name, profile_config) in &config.profiles {
        if profile_name != target_profile && profile_config.files.contains(&file_path.to_string()) {
            conflicts.push(profile_name.clone());
        }
    }
    conflicts
}

fn prompt_for_conflict_resolution(
    file_path: &str,
    conflicts: &[String],
    _target_profile: &str,
) -> bool {
    if conflicts.is_empty() {
        return true;
    }

    eprintln!("⚠️  Warning: File '{file_path}' already exists in other profiles:");
    for conflict in conflicts {
        eprintln!("   - {conflict}");
    }
    eprintln!("   This will create separate copies for each profile.");

    if atty::is(atty::Stream::Stdin) {
        eprint!("Continue? [y/N]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_lowercase();
        input == "y" || input == "yes"
    } else {
        eprintln!("[WARN] Non-interactive mode. Proceeding with separate copies.");
        true
    }
}

fn prompt_for_profile(profiles: &[&String], default_profile: &str) -> String {
    if profiles.is_empty() {
        eprintln!("No profiles are defined. Please add a profile first.");
        std::process::exit(1);
    }
    if profiles.len() == 1 {
        return profiles[0].clone();
    }
    // Check if stdin is a tty
    if atty::is(atty::Stream::Stdin) {
        eprintln!("Select a profile to add this file to:");
        for (i, profile) in profiles.iter().enumerate() {
            eprintln!("  {}. {}", i + 1, profile);
        }
        eprint!("Enter number (default: {default_profile}): ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input.is_empty() {
            return default_profile.to_string();
        }
        if let Ok(idx) = input.parse::<usize>() {
            if idx > 0 && idx <= profiles.len() {
                return profiles[idx - 1].clone();
            }
        }
        eprintln!("Invalid selection. Using default profile: {default_profile}");
        default_profile.to_string()
    } else {
        eprintln!("[WARN] No profile specified and not running interactively. Using default profile: {default_profile}");
        default_profile.to_string()
    }
}

fn color_enabled() -> bool {
    atty::is(atty::Stream::Stdout)
}

fn handle_missing_source_file(_file: &str, source_path: &Path, dest: &Path) -> anyhow::Result<()> {
    let msg = format!("Source file not found: {}", source_path.display());
    if color_enabled() {
        eprintln!("{}", msg.red());
        eprintln!(
            "{}",
            "This file may have been moved or deleted from the dotfiles repository.".yellow()
        );
        eprintln!(
            "{}",
            format!("Expected location: {}", source_path.display()).cyan()
        );
        eprintln!("{}", format!("Target location: {}", dest.display()).cyan());
        eprintln!(
            "{}",
            "Run 'ordinator add <file> --profile <profile>' to re-add the file.".yellow()
        );
    } else {
        eprintln!("{msg}");
        eprintln!("This file may have been moved or deleted from the dotfiles repository.");
        eprintln!("Expected location: {}", source_path.display());
        eprintln!("Target location: {}", dest.display());
        eprintln!("Run 'ordinator add <file> --profile <profile>' to re-add the file.");
    }
    Err(anyhow::anyhow!(
        "Source file not found: {}",
        source_path.display()
    ))
}

pub async fn run(args: Args) -> Result<()> {
    eprintln!("[DEBUG] args.verbose: {}", args.verbose);
    eprintln!(
        "[DEBUG] std::env::args: {:?}",
        std::env::args().collect::<Vec<_>>()
    );
    // Setup logging based on verbose flag
    let log_level = if args.verbose {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    // Only initialize if not already initialized
    let _ = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .try_init();

    info!("Starting Ordinator");

    if args.dry_run {
        warn!("Running in DRY-RUN mode - no changes will be made");
        eprintln!("DRY-RUN: No changes will be made");
    }

    match args.command {
        Commands::Init {
            repo_url,
            target_dir,
            profile,
            force,
        } => {
            if let Some(url) = &repo_url {
                // Validate the repository URL format first
                let target_path = if let Some(dir) = &target_dir {
                    PathBuf::from(dir)
                } else {
                    std::env::current_dir()?
                };
                let repo_manager = crate::repo::RepoManager::new(target_path.clone());
                if let Err(e) = repo_manager.parse_github_url(url) {
                    return Err(anyhow::anyhow!("Invalid GitHub URL '{}': {}", url, e));
                }
                // Try to clone first (existing repository)
                match repo_manager.init_from_url(url, force).await {
                    Ok(_) => {
                        // Successfully cloned existing repository
                        info!("Repository initialized from existing repository: {}", url);
                        if !args.quiet {
                            eprintln!("Repository initialized successfully from {url}");
                            eprintln!("Next steps:");
                            eprintln!("  1. Review the configuration: cat ordinator.toml");
                            eprintln!("  2. Apply the dotfiles: ordinator apply");
                            eprintln!("  3. Set up secrets (if needed): ordinator secrets setup");
                        }
                        Ok(())
                    }
                    Err(_) => {
                        // Failed to clone - treat as new repository URL
                        info!("Initializing new repository with remote URL: {}", url);
                        if !args.quiet {
                            eprintln!("Initializing new repository with remote URL: {url}");
                        }

                        if args.dry_run {
                            info!(
                                "[DRY RUN] Would initialize new repository with remote: {}",
                                url
                            );
                            eprintln!(
                                "DRY-RUN: Would initialize new repository with remote: {url}"
                            );
                            return Ok(());
                        }

                        // Initialize new repository
                        let config_path = Config::init_dotfiles_repository()?;
                        info!("Created configuration file: {}", config_path.display());
                        eprintln!("Created configuration file: {}", config_path.display());

                        // Initialize Git repository
                        let dotfiles_path = config_path.parent().unwrap().to_path_buf();
                        let git_manager = GitManager::new(dotfiles_path.clone());

                        if !git_manager.exists() {
                            git_manager.init()?;
                            info!("Git repository initialized at: {}", dotfiles_path.display());
                            eprintln!("Git repository initialized at: {}", dotfiles_path.display());
                        } else {
                            info!(
                                "Git repository already exists at: {}",
                                dotfiles_path.display()
                            );
                            eprintln!(
                                "Git repository already exists at: {}",
                                dotfiles_path.display()
                            );
                        }

                        // Set the remote URL
                        git_manager.add_remote("origin", url)?;
                        info!("Set remote 'origin' to: {}", url);
                        eprintln!("Set remote 'origin' to: {url}");

                        // Generate README with correct URL
                        let config = Config::from_file(&config_path)?;
                        if let Err(e) = crate::readme::auto_update_readme(&config, &dotfiles_path) {
                            if !args.quiet {
                                eprintln!("Warning: Failed to generate README: {e}");
                            }
                        } else {
                            eprintln!("Generated README.md with correct repository URL");
                        }

                        info!("Repository initialization completed");
                        eprintln!("Repository initialization completed");
                        eprintln!("Next steps:");
                        eprintln!(
                            "  1. Add your first file: ordinator add ~/.zshrc --profile work"
                        );
                        eprintln!("  2. Apply your configuration: ordinator apply --profile work");
                        eprintln!("  3. Commit and push: ordinator commit -m 'Initial setup' && ordinator push");
                        Ok(())
                    }
                }
            } else {
                // Initialize new repository (existing behavior)
                info!("Initializing new repository with profile: {}", profile);
                if !args.quiet {
                    eprintln!("Initializing new repository with profile: {profile}");
                }

                if args.dry_run {
                    info!(
                        "[DRY RUN] Would initialize repository with profile: {}",
                        profile
                    );
                    eprintln!("DRY-RUN: Would initialize repository with profile: {profile}");
                    return Ok(());
                }

                // Initialize the dotfiles repository
                let config_path = Config::init_dotfiles_repository()?;
                info!("Created configuration file: {}", config_path.display());
                eprintln!("Created configuration file: {}", config_path.display());

                // Initialize Git repository
                let dotfiles_path = config_path.parent().unwrap().to_path_buf();
                let git_manager = GitManager::new(dotfiles_path.clone());

                if !git_manager.exists() {
                    git_manager.init()?;
                    info!("Git repository initialized at: {}", dotfiles_path.display());
                    eprintln!("Git repository initialized at: {}", dotfiles_path.display());
                } else {
                    info!(
                        "Git repository already exists at: {}",
                        dotfiles_path.display()
                    );
                    eprintln!(
                        "Git repository already exists at: {}",
                        dotfiles_path.display()
                    );
                }

                info!("Repository initialization completed");
                eprintln!("Repository initialization completed");
                Ok(())
            }
        }
        Commands::Add { path, profile } => {
            let (mut config, config_path) = Config::load()?;
            let profile_name = match profile {
                Some(p) => p,
                None => {
                    let profiles = config.list_profiles();
                    prompt_for_profile(&profiles, &config.global.default_profile)
                }
            };
            if !config.profiles.contains_key(&profile_name) {
                return Err(anyhow::anyhow!(
                    "Profile '{}' does not exist. To create it, run: ordinator profile add {}",
                    profile_name,
                    profile_name
                ));
            }
            // Exclusion check
            let exclusion_set = config.exclusion_set_for_profile(&profile_name)?;
            if exclusion_set.is_match(&path) {
                return Err(anyhow::anyhow!(
                    "Path '{}' matches an exclusion pattern and cannot be tracked.",
                    path
                ));
            }
            if args.dry_run {
                println!("DRY-RUN: Would add '{path}' to profile '{profile_name}'");
                return Ok(());
            }
            let path_obj = Path::new(&path);
            if !path_obj.exists() {
                return Err(anyhow::anyhow!("Path '{}' does not exist on disk.", path));
            }

            // Check for conflicts with other profiles
            let conflicts = check_file_conflicts(&config, &path, &profile_name);
            if !conflicts.is_empty()
                && !prompt_for_conflict_resolution(&path, &conflicts, &profile_name)
            {
                eprintln!("Operation cancelled by user.");
                return Ok(());
            }

            // Get the profile-specific file path
            let profile_file_path = config.get_profile_file_path(&profile_name, &path)?;

            // Create the profile directory if it doesn't exist
            if let Some(parent) = profile_file_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Copy the file to the profile-specific location
            if path_obj.is_file() {
                std::fs::copy(path_obj, &profile_file_path)?;
                if !args.quiet {
                    let msg = format!("[1/1] Copied '{path}' to profile '{profile_name}' storage");
                    if color_enabled() {
                        println!("{}", msg.green());
                    } else {
                        println!("{msg}");
                    }
                }
            } else if path_obj.is_dir() {
                // For directories, we need to copy recursively
                let mut file_count = 0;
                for entry in walkdir::WalkDir::new(path_obj).into_iter().flatten() {
                    if entry.path().is_file() {
                        file_count += 1;
                    }
                }
                let mut copied = 0;
                for entry in walkdir::WalkDir::new(path_obj) {
                    let entry = entry?;
                    let src_path = entry.path();
                    if src_path.is_file() {
                        let rel_path = src_path.strip_prefix(path_obj).unwrap();
                        let dst_path = profile_file_path.join(rel_path);
                        if let Some(parent) = dst_path.parent() {
                            std::fs::create_dir_all(parent)?;
                        }
                        std::fs::copy(src_path, &dst_path)?;
                        copied += 1;
                        if !args.quiet && color_enabled() {
                            println!(
                                "[{} / {}] {}",
                                copied,
                                file_count,
                                dst_path.display().to_string().cyan()
                            );
                        } else if !args.quiet {
                            println!("[{} / {}] {}", copied, file_count, dst_path.display());
                        }
                    }
                }
                if !args.quiet {
                    let msg =
                        format!("Copied directory '{path}' to profile '{profile_name}' storage");
                    if color_enabled() {
                        println!("{}", msg.green());
                    } else {
                        println!("{msg}");
                    }
                }
            }

            config.add_file_to_profile(&profile_name, path.clone())?;
            config.save_to_file(&config_path)?;

            // Automatically scan the added file for secrets
            if path_obj.is_file() {
                let base_dir = config_path.parent().unwrap().to_path_buf();
                let manager = crate::secrets::SecretsManager::new(
                    None,
                    None,
                    config.clone(),
                    base_dir.clone(),
                );

                match manager.check_for_plaintext_secrets(path_obj) {
                    Ok(has_secrets) => {
                        if has_secrets {
                            eprintln!("⚠️  Warning: '{path}' contains potential secrets");
                            match manager.get_secrets_info(path_obj) {
                                Ok(secret_types) => {
                                    eprintln!("   Found: {}", secret_types.join(", "));
                                }
                                Err(_) => {
                                    eprintln!("   Found: potential secrets");
                                }
                            }
                            eprintln!(
                                "   Consider encrypting with: ordinator secrets encrypt {path}"
                            );
                            eprintln!("   Use 'ordinator commit --force' to commit anyway");
                        }
                    }
                    Err(e) => {
                        if args.verbose {
                            eprintln!("Warning: Could not scan '{path}' for secrets: {e}");
                        }
                    }
                }
            }

            println!("Added '{path}' to profile '{profile_name}'");

            // Auto-update README if needed
            if !args.dry_run {
                let dotfiles_dir = config_path.parent().unwrap();
                if crate::readme::readme_needs_update(&config) {
                    if config.readme.auto_update {
                        if let Err(e) = crate::readme::auto_update_readme(&config, dotfiles_dir) {
                            if !args.quiet {
                                eprintln!("Warning: Failed to auto-update README: {e}");
                            }
                        }
                    } else {
                        eprintln!(
                            "⚠️  Ordinator config changed ({}). Your README.md may be out of date.",
                            config.readme.update_on_changes.join(", ")
                        );
                        eprintln!(
                            "   Run: ordinator readme default   (or ordinator readme preview)"
                        );
                    }
                }
            }

            Ok(())
        }
        Commands::Remove { path } => {
            info!("Removing file: {}", path);
            eprintln!("Removing file: {path}");

            if args.dry_run {
                info!("[DRY RUN] Would remove file: {}", path);
                eprintln!("DRY-RUN: Would remove file: {path}");
                return Ok(());
            }

            // TODO: Implement actual remove logic
            info!("File removal not yet implemented");
            eprintln!("File removal not yet implemented");
            Ok(())
        }
        Commands::Commit { message, force } => {
            info!("Committing with message: {}", message);
            eprintln!("Committing with message: {message}");

            if args.dry_run {
                info!("[DRY RUN] Would commit with message: {}", message);
                eprintln!("DRY-RUN: Would commit with message: {message}");
                return Ok(());
            }

            if message.trim().is_empty() {
                eprintln!("Commit message cannot be empty.");
                std::process::exit(1);
            }

            // Load config and get dotfiles repo path
            let (config, config_path) = Config::load()?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();
            let git_manager = GitManager::new(dotfiles_path.clone());
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }

            // Scan for secrets before committing (unless --force is used)
            if !force {
                eprintln!("[DEBUG] Scanning for secrets before commit...");
                let base_dir = config_path.parent().unwrap().to_path_buf();
                let manager = crate::secrets::SecretsManager::new(
                    None,
                    None,
                    config.clone(),
                    base_dir.clone(),
                );

                let mut found_secrets = false;
                let mut files_with_secrets = Vec::new();

                // Scan all tracked files for secrets
                for profile in config.profiles.values() {
                    for file_path in &profile.files {
                        let full_path = base_dir.join(file_path);
                        eprintln!("[DEBUG] Scanning file: {full_path:?}");
                        if full_path.exists() && full_path.is_file() {
                            match manager.check_for_plaintext_secrets(&full_path) {
                                Ok(has_secrets) => {
                                    eprintln!(
                                        "[DEBUG] File {full_path:?} has secrets: {has_secrets}"
                                    );
                                    if has_secrets {
                                        found_secrets = true;
                                        files_with_secrets.push(file_path.clone());
                                        eprintln!(
                                            "⚠️  Warning: '{file_path}' contains potential secrets"
                                        );
                                        match manager.get_secrets_info(&full_path) {
                                            Ok(secret_types) => {
                                                eprintln!("   Found: {}", secret_types.join(", "));
                                            }
                                            Err(_) => {
                                                eprintln!("   Found: potential secrets");
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("[DEBUG] Error scanning file {full_path:?}: {e}");
                                    if args.verbose {
                                        eprintln!(
                                            "Warning: Could not scan '{file_path}' for secrets: {e}"
                                        );
                                    }
                                }
                            }
                        } else {
                            eprintln!(
                                "[DEBUG] File does not exist or is not a file: {full_path:?}"
                            );
                        }
                    }
                }

                eprintln!("[DEBUG] Found secrets: {found_secrets}");
                if found_secrets {
                    eprintln!("⚠️  Plaintext secrets detected in tracked files");
                    eprintln!("   Consider encrypting with: ordinator secrets encrypt <file>");
                    eprintln!("   Use --force to commit anyway");
                    std::process::exit(1);
                }
            }

            git_manager.commit(&message)?;
            info!("Changes committed successfully");
            eprintln!("Changes committed successfully");
            Ok(())
        }
        Commands::Push { repo_url, force } => {
            info!("Pushing changes{}", if force { " (force)" } else { "" });
            eprintln!("Pushing changes{}", if force { " (force)" } else { "" });

            if args.dry_run {
                info!(
                    "[DRY RUN] Would push changes{}",
                    if force { " (force)" } else { "" }
                );
                eprintln!(
                    "DRY-RUN: Would push changes{}",
                    if force { " (force)" } else { "" }
                );
                return Ok(());
            }

            // Load config and get dotfiles repo path
            let (_config, config_path) = Config::load()?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();
            let git_manager = GitManager::new(dotfiles_path.clone());
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }

            // If repository URL is provided, validate it first
            if let Some(url) = &repo_url {
                // Validate the repository URL format
                let repo_manager = crate::repo::RepoManager::new(dotfiles_path.clone());
                match repo_manager.parse_github_url(url) {
                    Ok(_) => {
                        info!("Setting remote 'origin' to: {}", url);
                        if !args.quiet {
                            eprintln!("Setting remote 'origin' to: {url}");
                        }
                        git_manager.add_remote("origin", url)?;
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Invalid repository URL '{}': {}", url, e));
                    }
                }
            }

            git_manager.push(force)?;
            info!("Changes pushed successfully");
            eprintln!("Changes pushed successfully");
            Ok(())
        }
        Commands::Pull { rebase } => {
            info!("Pulling changes{}", if rebase { " (rebase)" } else { "" });
            eprintln!("Pulling changes{}", if rebase { " (rebase)" } else { "" });

            if args.dry_run {
                info!(
                    "[DRY RUN] Would pull changes{}",
                    if rebase { " (rebase)" } else { "" }
                );
                eprintln!(
                    "DRY-RUN: Would pull changes{}",
                    if rebase { " (rebase)" } else { "" }
                );
                return Ok(());
            }

            // Load config and get dotfiles repo path
            let (_config, config_path) = Config::load()?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();
            let git_manager = GitManager::new(dotfiles_path.clone());
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }
            git_manager.pull(rebase)?;
            info!("Changes pulled successfully");
            eprintln!("Changes pulled successfully");
            Ok(())
        }
        Commands::Sync { force } => {
            info!("Syncing repository{}", if force { " (force)" } else { "" });
            eprintln!("Syncing repository{}", if force { " (force)" } else { "" });

            if args.dry_run {
                info!(
                    "[DRY RUN] Would sync repository{}",
                    if force { " (force)" } else { "" }
                );
                eprintln!(
                    "DRY-RUN: Would sync repository{}",
                    if force { " (force)" } else { "" }
                );
                return Ok(());
            }

            // Load config and get dotfiles repo path
            let (_config, config_path) = Config::load()?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();
            let git_manager = GitManager::new(dotfiles_path.clone());
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }
            // Pull first, then push
            git_manager.pull(false)?;
            git_manager.push(force)?;
            info!("Repository synced successfully");
            eprintln!("Repository synced successfully");
            Ok(())
        }
        Commands::Status { verbose } => {
            info!("Showing status{}", if verbose { " (verbose)" } else { "" });
            if !args.quiet {
                eprintln!("Showing status{}", if verbose { " (verbose)" } else { "" });
            }

            if args.dry_run {
                info!(
                    "[DRY RUN] Would show status{}",
                    if verbose { " (verbose)" } else { "" }
                );
                eprintln!(
                    "DRY-RUN: Would show status{}",
                    if verbose { " (verbose)" } else { "" }
                );
                return Ok(());
            }

            // Load config and get dotfiles repo path
            let (config, config_path) = Config::load()?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();
            let git_manager = GitManager::new(dotfiles_path.clone());

            // In test mode, treat .git dir as valid for status
            let is_test_mode = std::env::var("ORDINATOR_TEST_MODE").unwrap_or_default() == "1";
            let git_exists = dotfiles_path.join(".git").exists();
            if is_test_mode && git_exists {
                eprintln!("[TEST MODE] .git directory exists, simulating git status.");
            } else if git_manager.exists() {
                let status = git_manager.status()?;
                eprintln!("{status}");
            } else {
                eprintln!("No Git repository found. Showing symlink status only.");
            }

            // Show symlink status if verbose
            if verbose {
                eprintln!("\nSymlink Status:");
                use crate::utils::{get_home_dir, is_broken_symlink, is_symlink};
                let home_dir = get_home_dir()?;
                let _dotfiles_dir = config_path.parent().unwrap();
                let mut total_files = 0;
                let mut valid_symlinks = 0;
                let mut broken_symlinks = 0;
                let mut missing_files = 0;

                for profile_name in config.list_profiles() {
                    if let Some(profile_cfg) = config.get_profile(profile_name) {
                        eprintln!("  Profile: {profile_name}");
                        for file in &profile_cfg.files {
                            total_files += 1;
                            let dest = home_dir.join(file);

                            if !dest.exists() {
                                eprintln!("    {}: Missing", dest.display());
                                missing_files += 1;
                            } else if is_broken_symlink(&dest) {
                                eprintln!("    {}: Broken symlink", dest.display());
                                broken_symlinks += 1;
                            } else if is_symlink(&dest) {
                                eprintln!("    {}: Valid symlink", dest.display());
                                valid_symlinks += 1;
                            } else {
                                eprintln!("    {}: File (not symlinked)", dest.display());
                                missing_files += 1;
                            }
                        }
                    }
                }

                eprintln!("\nSummary:");
                eprintln!("  Total tracked files: {total_files}");
                eprintln!("  Valid symlinks: {valid_symlinks}");
                eprintln!("  Broken symlinks: {broken_symlinks}");
                eprintln!("  Missing/not symlinked: {missing_files}");
            }

            Ok(())
        }
        Commands::Apply {
            profile,
            skip_bootstrap,
            skip_secrets,
            skip_brew,
            force,
        } => {
            let (config, config_path) = Config::load()?;
            if !config.profiles.contains_key(&profile) {
                return Err(anyhow::anyhow!("Profile '{profile}' does not exist."));
            }
            info!("Applying profile: {}", profile);
            if !args.quiet {
                eprintln!("Applying profile: {profile}");
            }
            if skip_bootstrap {
                info!("Skipping bootstrap");
                if !args.quiet {
                    eprintln!("Skipping bootstrap");
                }
            }
            if skip_secrets {
                info!("Skipping secrets");
                if !args.quiet {
                    eprintln!("Skipping secrets");
                }
            }

            if args.dry_run {
                info!("[DRY RUN] Would apply profile: {}", profile);
                eprintln!("DRY-RUN: Would apply profile: {profile}");
                if skip_bootstrap {
                    info!("[DRY RUN] Would skip bootstrap");
                    eprintln!("DRY-RUN: Would skip bootstrap");
                }
                if skip_secrets {
                    info!("[DRY RUN] Would skip secrets");
                    eprintln!("DRY-RUN: Would skip secrets");
                }
                if skip_brew {
                    info!("[DRY RUN] Would skip brew");
                    eprintln!("DRY-RUN: Would skip brew");
                }
                return Ok(());
            }

            // Debug: print config information
            eprintln!("[DEBUG] Config loaded from: {}", config_path.display());
            eprintln!("[DEBUG] Requested profile: '{profile}'");
            eprintln!(
                "[DEBUG] Profile exists: {}",
                config.profiles.contains_key(&profile)
            );
            eprintln!("[DEBUG] Available profiles: {:?}", config.list_profiles());
            eprintln!(
                "[DEBUG] Profile files count: {}",
                config.get_profile(&profile).unwrap().files.len()
            );
            eprintln!(
                "[DEBUG] Profile files: {:?}",
                config.get_profile(&profile).unwrap().files
            );

            // Debug: print config file content
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    eprintln!("[DEBUG] Config file content:");
                    eprintln!("{content}");
                }
                Err(e) => {
                    eprintln!("[DEBUG] Failed to read config file: {e}");
                }
            }

            // For each tracked file, symlink with enhanced conflict resolution
            use crate::utils::{
                create_symlink_with_conflict_resolution, get_home_dir, get_symlink_target,
                is_symlink,
            };
            let home_dir = get_home_dir()?;
            let _dotfiles_dir = config_path.parent().unwrap();

            // Debug: print profile file list
            eprintln!(
                "[DEBUG] Profile '{}' has {} files:",
                profile,
                config.get_profile(&profile).unwrap().files.len()
            );
            for file in &config.get_profile(&profile).unwrap().files {
                eprintln!("[DEBUG]   - {file}");
            }

            for file in &config.get_profile(&profile).unwrap().files {
                // Get the source file path (profile-specific or fallback to flat structure)
                let source_path = config.get_source_file_path(&profile, file)?;
                let dest = home_dir.join(file);

                eprintln!("[DEBUG] Checking file: {file}");
                eprintln!("[DEBUG] Source: {}", source_path.display());
                eprintln!("[DEBUG] Dest: {}", dest.display());

                if !source_path.exists() {
                    return handle_missing_source_file(file, &source_path, &dest);
                }

                if !dest.exists() {
                    // Create new symlink
                    let msg = format!(
                        "[{}] Symlinking {} -> {}",
                        file,
                        dest.display(),
                        source_path.display()
                    );
                    if !args.quiet && color_enabled() {
                        println!("{}", msg.cyan());
                    } else if !args.quiet {
                        println!("{msg}");
                    }
                    if args.dry_run {
                        let msg = format!(
                            "DRY-RUN: Would create symlink {} -> {}",
                            dest.display(),
                            source_path.display()
                        );
                        if color_enabled() {
                            println!("{}", msg.yellow());
                        } else {
                            println!("{msg}");
                        }
                    } else {
                        create_symlink_with_conflict_resolution(
                            &source_path,
                            &dest,
                            force,
                            config.global.create_backups,
                            &config_path,
                        )?;
                        if !args.quiet {
                            let msg = format!(
                                "Symlinked: {} -> {}",
                                dest.display(),
                                source_path.display()
                            );
                            if color_enabled() {
                                println!("{}", msg.green());
                            } else {
                                println!("{msg}");
                            }
                        }
                    }
                    continue;
                }

                if !is_symlink(&dest) {
                    // Handle non-symlink conflict
                    let msg = format!(
                        "Conflict: {} already exists and is not a symlink",
                        dest.display()
                    );
                    if color_enabled() {
                        eprintln!("{}", msg.red());
                        eprintln!(
                            "{}",
                            "Use --force to overwrite, or manually remove the file first.".yellow()
                        );
                    } else {
                        eprintln!("{msg}");
                        eprintln!("Use --force to overwrite, or manually remove the file first.");
                    }
                    if !force {
                        return Err(anyhow::anyhow!(
                            "Target {} already exists and is not a symlink. Use --force to overwrite.",
                            dest.display()
                        ));
                    }
                    // Force overwrite - create symlink
                    let msg = format!(
                        "Force creating symlink: {} -> {}",
                        dest.display(),
                        source_path.display()
                    );
                    if !args.quiet && color_enabled() {
                        println!("{}", msg.yellow());
                    } else if !args.quiet {
                        println!("{msg}");
                    }
                    if args.dry_run {
                        let msg = format!(
                            "DRY-RUN: Would force create symlink {} -> {}",
                            dest.display(),
                            source_path.display()
                        );
                        if color_enabled() {
                            println!("{}", msg.yellow());
                        } else {
                            println!("{msg}");
                        }
                    } else {
                        create_symlink_with_conflict_resolution(
                            &source_path,
                            &dest,
                            force,
                            config.global.create_backups,
                            &config_path,
                        )?;
                        if !args.quiet {
                            let msg = format!(
                                "Symlinked: {} -> {}",
                                dest.display(),
                                source_path.display()
                            );
                            if color_enabled() {
                                println!("{}", msg.green());
                            } else {
                                println!("{msg}");
                            }
                        }
                    }
                    continue;
                }

                // Check if existing symlink is broken or points to wrong target
                let needs_repair = match get_symlink_target(&dest) {
                    Ok(actual_target) => {
                        eprintln!("[DEBUG] actual_target: {}", actual_target.display());
                        eprintln!("[DEBUG] expected source: {}", source_path.display());
                        eprintln!("[DEBUG] actual_target.exists(): {}", actual_target.exists());
                        let needs = actual_target != source_path || !actual_target.exists();
                        eprintln!("[DEBUG] needs_repair: {needs}");
                        needs
                    }
                    Err(e) => {
                        eprintln!(
                            "[DEBUG] Could not read symlink target for {}: {}",
                            dest.display(),
                            e
                        );
                        true // Can't read symlink target, assume broken
                    }
                };

                eprintln!(
                    "[DEBUG] After needs_repair check: {} => {}",
                    dest.display(),
                    needs_repair
                );

                if needs_repair {
                    eprintln!("[DEBUG] Entering repair branch for {}", dest.display());
                    if args.dry_run {
                        eprintln!("DRY-RUN: Would repair {}", dest.display());
                    } else {
                        use crate::utils::repair_symlink;
                        repair_symlink(&dest, &source_path)?;
                        if !args.quiet {
                            eprintln!("Repaired: {} -> {}", dest.display(), source_path.display());
                        }
                    }
                } else if args.verbose {
                    eprintln!("  {}: Valid symlink", dest.display());
                }
            }

            // Generate bootstrap script if not skipped
            if !skip_bootstrap {
                use crate::bootstrap::BootstrapManager;
                let bootstrap_manager = BootstrapManager::new(args.dry_run);

                if let Some(script_path) =
                    bootstrap_manager.generate_bootstrap_script(&profile, &config, _dotfiles_dir)?
                {
                    let safety_level = bootstrap_manager.get_script_safety_level(&script_path);

                    if !args.quiet {
                        eprintln!("Generated bootstrap script: {}", script_path.display());
                        match safety_level {
                            crate::bootstrap::SafetyLevel::Safe => {
                                eprintln!("Script safety: Safe");
                            }
                            crate::bootstrap::SafetyLevel::Warning => {
                                eprintln!("Script safety: Warning - Review before execution");
                            }
                            crate::bootstrap::SafetyLevel::Dangerous => {
                                eprintln!(
                                    "Script safety: Dangerous - Review carefully before execution"
                                );
                            }
                            crate::bootstrap::SafetyLevel::Blocked => {
                                eprintln!(
                                    "Script safety: Blocked - Script contains dangerous commands"
                                );
                            }
                        }
                    }

                    info!(
                        "Generated bootstrap script: {} (safety: {:?})",
                        script_path.display(),
                        safety_level
                    );
                } else {
                    if !args.quiet {
                        eprintln!("No bootstrap script defined for profile '{profile}'");
                    }
                    info!("No bootstrap script defined for profile '{}'", profile);
                }
            } else {
                info!("Skipped bootstrap script generation");
            }

            // Install Homebrew packages if not skipped
            if !skip_brew {
                use crate::brew::BrewManager;

                // Check if Homebrew is installed
                if BrewManager::check_homebrew_installed() {
                    let brew_manager = BrewManager::new(args.dry_run);

                    if !args.quiet {
                        eprintln!("Installing Homebrew packages for profile '{profile}'");
                    }

                    if let Err(e) = brew_manager.install_packages(&profile, &config).await {
                        if !args.quiet {
                            eprintln!("Warning: Failed to install Homebrew packages: {e}");
                        }
                    } else if !args.quiet {
                        eprintln!("✅ Homebrew packages installed successfully");
                    }
                } else {
                    if !args.quiet {
                        eprintln!("⚠️  Homebrew not installed - skipping package installation");
                    }
                    info!("Homebrew not installed, skipping package installation");
                }
            } else {
                info!("Skipped Homebrew package installation");
                if !args.quiet {
                    eprintln!("Skipped Homebrew package installation");
                }
            }

            info!("Apply completed");
            if !args.quiet {
                eprintln!("Apply completed");
            }

            // Auto-update README if needed
            if !args.dry_run && crate::readme::readme_needs_update(&config) {
                if config.readme.auto_update {
                    if let Err(e) = crate::readme::auto_update_readme(&config, _dotfiles_dir) {
                        if !args.quiet {
                            eprintln!("Warning: Failed to auto-update README: {e}");
                        }
                    }
                } else {
                    eprintln!(
                        "⚠️  Ordinator config changed ({}). Your README.md may be out of date.",
                        config.readme.update_on_changes.join(", ")
                    );
                    eprintln!("   Run: ordinator readme default   (or ordinator readme preview)");
                }
            }

            Ok(())
        }
        Commands::Repair { profile, verbose } => {
            info!("Repairing broken symlinks");
            if !args.quiet {
                eprintln!("Repairing broken symlinks");
            }

            if args.dry_run {
                info!("[DRY RUN] Would repair broken symlinks");
                eprintln!("DRY-RUN: Would repair broken symlinks");
                return Ok(());
            }

            // Load config
            let (config, config_path) = Config::load()?;
            use crate::utils::{get_home_dir, get_symlink_target, is_symlink};
            let home_dir = get_home_dir()?;
            let _dotfiles_dir = config_path.parent().unwrap();

            let profiles_to_repair = if let Some(profile_name) = profile {
                vec![profile_name]
            } else {
                config
                    .list_profiles()
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect()
            };

            let mut total_checked = 0;
            let mut total_repaired = 0;

            for profile_name in &profiles_to_repair {
                if let Some(profile_cfg) = config.get_profile(profile_name) {
                    if verbose {
                        eprintln!("Checking profile: {profile_name}");
                    }

                    for file in &profile_cfg.files {
                        total_checked += 1;
                        let dest = home_dir.join(file);

                        eprintln!("[DEBUG] Checking file: {file}");
                        eprintln!("[DEBUG] Dest: {}", dest.display());
                        eprintln!("[DEBUG] home_dir: {}", home_dir.display());
                        eprintln!("[DEBUG] dotfiles_dir: {}", _dotfiles_dir.display());

                        // Check if destination exists or is a symlink (even if broken)
                        if !dest.exists() && !is_symlink(&dest) {
                            eprintln!(
                                "[DEBUG] Destination does not exist and is not a symlink: {}",
                                dest.display()
                            );
                            continue; // File doesn't exist and is not a symlink, nothing to repair
                        }

                        if !is_symlink(&dest) {
                            eprintln!("[DEBUG] Not a symlink: {}", dest.display());
                            if verbose {
                                eprintln!("  {}: Not a symlink (skipping)", dest.display());
                            }
                            continue;
                        }
                        eprintln!("[DEBUG] Is a symlink, proceeding to check target");

                        // Check if symlink is broken or points to wrong target
                        eprintln!(
                            "[DEBUG] About to check symlink target for: {}",
                            dest.display()
                        );
                        let needs_repair = match get_symlink_target(&dest) {
                            Ok(actual_target) => {
                                eprintln!("[DEBUG] actual_target: {}", actual_target.display());
                                eprintln!(
                                    "[DEBUG] expected source: {}",
                                    _dotfiles_dir.join("files").join(file).display()
                                );
                                eprintln!(
                                    "[DEBUG] actual_target.exists(): {}",
                                    actual_target.exists()
                                );
                                let needs = actual_target != _dotfiles_dir.join("files").join(file)
                                    || !actual_target.exists();
                                eprintln!("[DEBUG] needs_repair: {needs}");
                                needs
                            }
                            Err(e) => {
                                eprintln!(
                                    "[DEBUG] Could not read symlink target for {}: {}",
                                    dest.display(),
                                    e
                                );
                                true // Can't read symlink target, assume broken
                            }
                        };

                        eprintln!(
                            "[DEBUG] After needs_repair check: {} => {}",
                            dest.display(),
                            needs_repair
                        );

                        if needs_repair {
                            eprintln!("[DEBUG] Entering repair branch for {}", dest.display());
                            if args.dry_run {
                                eprintln!("DRY-RUN: Would repair {}", dest.display());
                            } else {
                                use crate::utils::repair_symlink;
                                repair_symlink(&dest, &_dotfiles_dir.join("files").join(file))?;
                                if !args.quiet {
                                    eprintln!(
                                        "Repaired: {} -> {}",
                                        dest.display(),
                                        _dotfiles_dir.join("files").join(file).display()
                                    );
                                }
                                total_repaired += 1;
                            }
                        } else if verbose {
                            eprintln!("  {}: Valid symlink", dest.display());
                        }
                    }
                }
            }

            if !args.quiet {
                eprintln!("Repair completed: {total_checked} checked, {total_repaired} repaired");
            }
            info!("Repair completed: {total_checked} checked, {total_repaired} repaired");
            Ok(())
        }
        Commands::Profiles { verbose } => {
            info!(
                "Listing profiles{}",
                if verbose { " (verbose)" } else { "" }
            );
            eprintln!(
                "Listing profiles{}",
                if verbose { " (verbose)" } else { "" }
            );

            if args.dry_run {
                info!(
                    "[DRY RUN] Would list profiles{}",
                    if verbose { " (verbose)" } else { "" }
                );
                eprintln!(
                    "DRY-RUN: Would list profiles{}",
                    if verbose { " (verbose)" } else { "" }
                );
                return Ok(());
            }

            // Load and display profiles
            let (config, _config_path) = Config::load()?;
            let profiles = config.list_profiles();
            if profiles.is_empty() {
                return Err(anyhow::anyhow!(
                    "No profiles found in configuration. Run 'ordinator init' first."
                ));
            }
            eprintln!("Available profiles:");
            for profile_name in profiles {
                if let Some(profile) = config.get_profile(profile_name) {
                    eprintln!(
                        "  {}: {}",
                        profile_name,
                        profile.description.as_deref().unwrap_or("No description")
                    );
                }
            }
            Ok(())
        }
        Commands::Secrets { subcommand } => match subcommand {
            SecretCommands::Encrypt { file } => {
                let (config, config_path) = Config::load()?;
                let base_dir = config_path.parent().unwrap().to_path_buf();
                let mut manager = crate::secrets::SecretsManager::new(None, None, config, base_dir);
                let file_path = std::path::Path::new(&file);
                if !file_path.exists() {
                    eprintln!("Encryption failed: File '{file}' does not exist.");
                    std::process::exit(1);
                }
                match manager.encrypt_file(file_path) {
                    Ok(()) => {
                        println!("File encrypted successfully: {file}");
                    }
                    Err(e) => {
                        eprintln!("Encryption failed: {e}");
                        std::process::exit(1);
                    }
                }
                Ok(())
            }
            SecretCommands::Decrypt { file } => {
                let (config, config_path) = Config::load()?;
                let base_dir = config_path.parent().unwrap().to_path_buf();
                let mut manager = crate::secrets::SecretsManager::new(None, None, config, base_dir);
                let file_path = std::path::Path::new(&file);
                if !file_path.exists() {
                    eprintln!("Decryption failed: File '{file}' does not exist.");
                    std::process::exit(1);
                }
                match manager.decrypt_file(file_path) {
                    Ok(()) => {
                        println!("File decrypted successfully: {file}");
                    }
                    Err(e) => {
                        eprintln!("Decryption failed: {e}");
                        std::process::exit(1);
                    }
                }
                Ok(())
            }
            SecretCommands::List { paths_only } => {
                let (config, config_path) = Config::load()?;
                let base_dir = config_path.parent().unwrap().to_path_buf();
                let manager = crate::secrets::SecretsManager::new(None, None, config, base_dir);
                let files = manager.list_encrypted_files()?;
                if files.is_empty() {
                    println!("No files match the encryption patterns.");
                } else if paths_only {
                    for (path, _) in files {
                        println!("{}", path.display());
                    }
                } else {
                    println!("{:<50} | Status", "File");
                    println!("{}", "-".repeat(50));
                    for (path, encrypted) in files {
                        let status = if encrypted { "Encrypted" } else { "Plaintext" };
                        println!("{:<50} | {}", path.display(), status);
                    }
                }
                Ok(())
            }
            SecretCommands::Check => {
                use crate::secrets::check_sops_and_age;
                match check_sops_and_age() {
                    Ok(()) => {
                        println!("SOPS and age are both installed and available in PATH.");
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                }
                Ok(())
            }
            SecretCommands::Setup { profile, force } => {
                use crate::secrets::setup_sops_and_age;
                // Check for valid profile name (no slashes, etc.)
                if profile.contains('/') || profile.contains('\\') {
                    eprintln!("Setup failed: Invalid profile name '{profile}'");
                    std::process::exit(1);
                }
                match setup_sops_and_age(&profile, force) {
                    Ok(()) => {
                        println!(
                            "✅ SOPS and age setup completed successfully for profile: {profile}"
                        );
                    }
                    Err(e) => {
                        eprintln!("Setup failed: {e}");
                        std::process::exit(1);
                    }
                }
                Ok(())
            }
            SecretCommands::Scan { profile, verbose } => {
                let (config, config_path) = Config::load()?;
                let base_dir = config_path.parent().unwrap().to_path_buf();
                let manager = crate::secrets::SecretsManager::new(
                    None,
                    None,
                    config.clone(),
                    base_dir.clone(),
                );

                let profiles_to_scan = if let Some(profile_name) = profile {
                    if !config.profiles.contains_key(&profile_name) {
                        eprintln!("Scan failed: Profile '{profile_name}' does not exist.");
                        std::process::exit(1);
                    }
                    vec![profile_name]
                } else {
                    config
                        .list_profiles()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect()
                };

                let mut found_secrets = false;
                let mut total_files_scanned = 0;

                for profile_name in profiles_to_scan {
                    if let Some(profile) = config.get_profile(&profile_name) {
                        if !args.quiet {
                            eprintln!("Scanning profile: {profile_name}");
                        }

                        for file_path in &profile.files {
                            total_files_scanned += 1;
                            let full_path = base_dir.join(file_path);

                            if full_path.exists() && full_path.is_file() {
                                match manager.check_for_plaintext_secrets(&full_path) {
                                    Ok(has_secrets) => {
                                        if has_secrets {
                                            found_secrets = true;
                                            if verbose {
                                                // Get detailed info about what types of secrets were found
                                                match manager.get_secrets_info(&full_path) {
                                                    Ok(secret_types) => {
                                                        eprintln!("⚠️  Potential secrets found in: {} ({})", 
                                                                 file_path, secret_types.join(", "));
                                                    }
                                                    Err(_) => {
                                                        eprintln!(
                                                            "⚠️  Potential secrets found in: {file_path}"
                                                        );
                                                    }
                                                }
                                            } else {
                                                eprintln!("⚠️  {file_path}");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        if verbose {
                                            eprintln!("Error scanning {file_path}: {e}");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !found_secrets {
                    if !args.quiet {
                        eprintln!("✅ No plaintext secrets found in {total_files_scanned} files");
                    }
                } else {
                    if !args.quiet {
                        eprintln!("⚠️  Plaintext secrets detected in tracked files. Consider encrypting them with 'ordinator secrets encrypt <file>'");
                    }
                    // Always exit with error code when secrets are found
                    std::process::exit(1);
                }
                Ok(())
            }
        },
        Commands::GenerateScript { output, profile } => {
            info!(
                "Generating system script: {} for profile: {}",
                output, profile
            );
            eprintln!("Generating system script: {output} for profile: {profile}");

            if args.dry_run {
                info!(
                    "[DRY RUN] Would generate system script: {} for profile: {}",
                    output, profile
                );
                eprintln!("DRY-RUN: Would generate system script: {output} for profile: {profile}");
                return Ok(());
            }

            // TODO: Implement actual script generation logic
            info!("Script generation not yet implemented");
            eprintln!("Script generation not yet implemented");
            Ok(())
        }
        Commands::Bootstrap { profile, edit } => {
            info!("Bootstrap script info for profile: {}", profile);
            if !args.quiet {
                eprintln!("Bootstrap script info for profile: {profile}");
            }

            let (config, config_path) = Config::load()?;
            if !config.profiles.contains_key(&profile) {
                return Err(anyhow::anyhow!("Profile '{profile}' does not exist."));
            }

            let dotfiles_dir = config_path.parent().unwrap();
            use crate::bootstrap::{BootstrapManager, SafetyLevel};
            let bootstrap_manager = BootstrapManager::new(args.dry_run);

            // Generate the bootstrap script first
            if let Some(script_path) =
                bootstrap_manager.generate_bootstrap_script(&profile, &config, dotfiles_dir)?
            {
                let safety_level = bootstrap_manager.get_script_safety_level(&script_path);

                if !args.quiet {
                    eprintln!("Bootstrap script: {}", script_path.display());
                    eprintln!("Safety level: {safety_level:?}");
                }

                // Print warnings for dangerous/blocked scripts
                match safety_level {
                    SafetyLevel::Blocked => {
                        eprintln!("❌ Script is BLOCKED: Contains extremely dangerous commands (e.g., rm -rf /). Review and edit the script before running.");
                    }
                    SafetyLevel::Dangerous => {
                        eprintln!("⚠️  Script is DANGEROUS: Contains commands like 'sudo'. Review carefully before running.");
                    }
                    SafetyLevel::Warning => {
                        eprintln!("⚠️  Script contains potentially risky commands. Review before running.");
                    }
                    SafetyLevel::Safe => {
                        eprintln!("Script is marked as safe.");
                    }
                }

                if edit {
                    // Open the script in $EDITOR or nano
                    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
                    let status = std::process::Command::new(editor)
                        .arg(&script_path)
                        .status();
                    match status {
                        Ok(status) if status.success() => {
                            eprintln!("Script opened for editing: {}", script_path.display());
                        }
                        Ok(status) => {
                            eprintln!("Editor exited with status: {status}");
                        }
                        Err(e) => {
                            eprintln!("Failed to open editor: {e}");
                        }
                    }
                } else {
                    // Always print the command for the user to run
                    eprintln!("\nTo run the bootstrap script, execute:");
                    eprintln!("  bash {}", script_path.display());
                    eprintln!("\nOr review and edit the script before running as needed.");
                }
                info!(
                    "Bootstrap script info presented to user: {}",
                    script_path.display()
                );
            } else {
                if !args.quiet {
                    eprintln!("No bootstrap script defined for profile '{profile}'");
                }
                info!("No bootstrap script defined for profile '{}'", profile);
            }

            Ok(())
        }
        Commands::Brew { subcommand } => {
            use crate::brew::BrewManager;

            // Check if Homebrew is installed
            if !BrewManager::check_homebrew_installed() {
                eprintln!("❌ Homebrew is not installed. Please install Homebrew first:");
                eprintln!("   /bin/bash -c \"$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\"");
                std::process::exit(1);
            }

            match subcommand {
                BrewCommands::Export {
                    profile,
                    with_versions: _,
                    force,
                } => {
                    info!("Exporting Homebrew packages for profile: {}", profile);
                    if !args.quiet {
                        eprintln!("Exporting Homebrew packages for profile: {profile}");
                    }

                    let (mut config, config_path) = Config::load()?;

                    if !config.profiles.contains_key(&profile) {
                        return Err(anyhow::anyhow!("Profile '{}' does not exist.", profile));
                    }

                    // Check if packages already exist and force is not set
                    let profile_config = config.get_profile(&profile).unwrap();
                    if !profile_config.homebrew_packages.is_empty() && !force {
                        eprintln!("⚠️  Profile '{profile}' already has Homebrew packages defined.");
                        eprintln!("   Use --force to overwrite existing package list.");
                        std::process::exit(1);
                    }

                    if args.dry_run {
                        info!(
                            "[DRY RUN] Would export Homebrew packages for profile: {}",
                            profile
                        );
                        eprintln!("DRY-RUN: Would export Homebrew packages for profile: {profile}");
                        return Ok(());
                    }

                    let brew_manager = BrewManager::new(args.dry_run);
                    brew_manager.export_packages(&profile, &mut config).await?;

                    config.save_to_file(&config_path)?;

                    if !args.quiet {
                        eprintln!("✅ Exported Homebrew packages to profile '{profile}'");
                    }

                    // Auto-update README if needed
                    if !args.dry_run {
                        let dotfiles_dir = config_path.parent().unwrap();
                        if crate::readme::readme_needs_update(&config) {
                            if config.readme.auto_update {
                                if let Err(e) =
                                    crate::readme::auto_update_readme(&config, dotfiles_dir)
                                {
                                    if !args.quiet {
                                        eprintln!("Warning: Failed to auto-update README: {e}");
                                    }
                                }
                            } else {
                                eprintln!("⚠️  Ordinator config changed ({}). Your README.md may be out of date.", config.readme.update_on_changes.join(", "));
                                eprintln!("   Run: ordinator readme default   (or ordinator readme preview)");
                            }
                        }
                    }

                    Ok(())
                }
                BrewCommands::Install {
                    profile,
                    non_interactive: _,
                    force: _,
                } => {
                    info!("Installing Homebrew packages for profile: {}", profile);
                    if !args.quiet {
                        eprintln!("Installing Homebrew packages for profile: {profile}");
                    }

                    let (config, _) = Config::load()?;

                    if !config.profiles.contains_key(&profile) {
                        return Err(anyhow::anyhow!("Profile '{}' does not exist.", profile));
                    }

                    let brew_manager = BrewManager::new(args.dry_run);
                    brew_manager.install_packages(&profile, &config).await?;

                    if !args.quiet {
                        eprintln!(
                            "✅ Homebrew package installation complete for profile '{profile}'"
                        );
                    }

                    Ok(())
                }
                BrewCommands::List {
                    profile,
                    verbose: _,
                } => {
                    info!("Listing Homebrew packages for profile: {}", profile);
                    if !args.quiet {
                        eprintln!("Listing Homebrew packages for profile: {profile}");
                    }

                    let (config, _) = Config::load()?;

                    if !config.profiles.contains_key(&profile) {
                        return Err(anyhow::anyhow!("Profile '{}' does not exist.", profile));
                    }

                    let brew_manager = BrewManager::new(args.dry_run);
                    brew_manager.list_packages(&profile, &config)?;

                    Ok(())
                }
            }
        }
        Commands::Readme { subcommand } => {
            match subcommand {
                ReadmeCommands::Default => {
                    info!("Generating default README.md");
                    if !args.quiet {
                        eprintln!("Generating default README.md");
                    }

                    let (config, config_path) = Config::load()?;
                    let dotfiles_dir = config_path.parent().unwrap();
                    let readme_manager = crate::readme::ReadmeManager::new(args.dry_run);

                    if let Some(readme_path) =
                        readme_manager.generate_default_readme(&config, dotfiles_dir)?
                    {
                        if !args.quiet {
                            eprintln!("Generated default README.md: {}", readme_path.display());
                        }
                        info!("Generated default README.md: {}", readme_path.display());
                    } else {
                        if !args.quiet {
                            eprintln!("No default README.md found to generate.");
                        }
                        info!("No default README.md found to generate.");
                    }
                }
                ReadmeCommands::Interactive => {
                    info!("Interactive README customization");
                    if !args.quiet {
                        eprintln!("Interactive README customization");
                    }

                    let (config, config_path) = Config::load()?;
                    let dotfiles_dir = config_path.parent().unwrap();
                    let readme_manager = crate::readme::ReadmeManager::new(args.dry_run);

                    if let Some(readme_path) =
                        readme_manager.interactive_customization(&config, dotfiles_dir)?
                    {
                        if !args.quiet {
                            eprintln!(
                                "Interactive README customization complete. Generated: {}",
                                readme_path.display()
                            );
                        }
                        info!(
                            "Interactive README customization complete. Generated: {}",
                            readme_path.display()
                        );
                    } else {
                        if !args.quiet {
                            eprintln!("Interactive README customization cancelled or failed.");
                        }
                        info!("Interactive README customization cancelled or failed.");
                    }
                }
                ReadmeCommands::Preview => {
                    info!("Previewing generated README");
                    if !args.quiet {
                        eprintln!("Previewing generated README");
                    }

                    let (config, config_path) = Config::load()?;
                    let dotfiles_dir = config_path.parent().unwrap();
                    let readme_manager = crate::readme::ReadmeManager::new(args.dry_run);

                    if let Some(readme_path) =
                        readme_manager.preview_readme(&config, dotfiles_dir)?
                    {
                        if !args.quiet {
                            eprintln!("Previewing README: {}", readme_path.display());
                        }
                        info!("Previewing README: {}", readme_path.display());
                    } else {
                        if !args.quiet {
                            eprintln!("No README.md found to preview.");
                        }
                        info!("No README.md found to preview.");
                    }
                }
                ReadmeCommands::Edit => {
                    info!("Editing existing README.md");
                    if !args.quiet {
                        eprintln!("Editing existing README.md");
                    }

                    let (config, config_path) = Config::load()?;
                    let dotfiles_dir = config_path.parent().unwrap();
                    let readme_manager = crate::readme::ReadmeManager::new(args.dry_run);

                    if let Some(readme_path) = readme_manager.edit_readme(&config, dotfiles_dir)? {
                        if !args.quiet {
                            eprintln!("README.md edited: {}", readme_path.display());
                        }
                        info!("README.md edited: {}", readme_path.display());
                    } else {
                        if !args.quiet {
                            eprintln!("No README.md found to edit.");
                        }
                        info!("No README.md found to edit.");
                    }
                }
            }
            Ok(())
        }
    }
}
