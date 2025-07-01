use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{info, warn};
use std::path::Path;

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
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new dotfiles repository
    Init {
        /// Remote Git repository URL
        #[arg(long)]
        remote: Option<String>,

        /// Profile to use for initialization
        #[arg(long, default_value = "default")]
        profile: String,
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
    },

    /// Push changes to remote repository
    Push {
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
}

pub async fn run(args: Args) -> Result<()> {
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
        Commands::Init { remote, profile } => {
            info!("Initializing repository with profile: {}", profile);
            eprintln!("Initializing repository with profile: {profile}");
            if let Some(url) = &remote {
                info!("Remote URL: {}", url);
                eprintln!("Remote URL: {url}");
            }

            if args.dry_run {
                info!(
                    "[DRY RUN] Would initialize repository with profile: {}",
                    profile
                );
                eprintln!("DRY-RUN: Would initialize repository with profile: {profile}");
                if let Some(url) = remote {
                    info!("[DRY RUN] Would set remote URL: {}", url);
                    eprintln!("DRY-RUN: Would set remote URL: {url}");
                }
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

            // Add remote if provided
            if let Some(url) = remote {
                git_manager.add_remote("origin", &url)?;
                info!("Remote 'origin' added: {}", url);
                eprintln!("Remote 'origin' added: {url}");
            }

            info!("Repository initialization completed");
            eprintln!("Repository initialization completed");
            return Ok(());
        }
        Commands::Add { path, profile } => {
            let config_path = Config::find_config_file()?.ok_or_else(|| anyhow::anyhow!("No configuration file found. Please run 'ordinator init' first."))?;
            let mut config = Config::from_file(&config_path)?;
            let profile_name = profile.unwrap_or_else(|| config.global.default_profile.clone());
            if !config.profiles.contains_key(&profile_name) {
                return Err(anyhow::anyhow!(
                    "Profile '{}' does not exist. To create it, run: ordinator profile add {}",
                    profile_name, profile_name
                ));
            }
            // Exclusion check
            let exclusion_set = config.exclusion_set_for_profile(&profile_name)?;
            if exclusion_set.is_match(&path) {
                return Err(anyhow::anyhow!("Path '{}' matches an exclusion pattern and cannot be tracked.", path));
            }
            if args.dry_run {
                println!("DRY-RUN: Would add '{}' to profile '{}'", path, profile_name);
                return Ok(());
            }
            let path_obj = Path::new(&path);
            if !path_obj.exists() {
                return Err(anyhow::anyhow!("Path '{}' does not exist on disk.", path));
            }
            config.add_file_to_profile(&profile_name, path.clone())?;
            config.save_to_file(&config_path)?;
            println!("Added '{}' to profile '{}'", path, profile_name);
            return Ok(());
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
            return Ok(());
        }
        Commands::Commit { message } => {
            info!("Committing with message: {}", message);
            eprintln!("Committing with message: {message}");

            if args.dry_run {
                info!("[DRY RUN] Would commit with message: {}", message);
                eprintln!("DRY-RUN: Would commit with message: {message}");
                return Ok(());
            }

            // Get the dotfiles repository path
            let config_path = Config::find_config_file()
                .with_context(|| "No configuration file found. Run 'ordinator init' first.")?
                .ok_or_else(|| {
                    anyhow::anyhow!("No configuration file found. Run 'ordinator init' first.")
                })?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();

            let git_manager = GitManager::new(dotfiles_path);
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }

            git_manager.commit(&message)?;
            info!("Changes committed successfully");
            eprintln!("Changes committed successfully");
            return Ok(());
        }
        Commands::Push { force } => {
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

            // Get the dotfiles repository path
            let config_path = Config::find_config_file()
                .with_context(|| "No configuration file found. Run 'ordinator init' first.")?
                .ok_or_else(|| {
                    anyhow::anyhow!("No configuration file found. Run 'ordinator init' first.")
                })?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();

            let git_manager = GitManager::new(dotfiles_path);
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }

            git_manager.push(force)?;
            info!("Changes pushed successfully");
            eprintln!("Changes pushed successfully");
            return Ok(());
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

            // Get the dotfiles repository path
            let config_path = Config::find_config_file()
                .with_context(|| "No configuration file found. Run 'ordinator init' first.")?
                .ok_or_else(|| {
                    anyhow::anyhow!("No configuration file found. Run 'ordinator init' first.")
                })?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();

            let git_manager = GitManager::new(dotfiles_path);
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }

            git_manager.pull(rebase)?;
            info!("Changes pulled successfully");
            eprintln!("Changes pulled successfully");
            return Ok(());
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

            // Get the dotfiles repository path
            let config_path = Config::find_config_file()
                .with_context(|| "No configuration file found. Run 'ordinator init' first.")?
                .ok_or_else(|| {
                    anyhow::anyhow!("No configuration file found. Run 'ordinator init' first.")
                })?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();

            let git_manager = GitManager::new(dotfiles_path);
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
            return Ok(());
        }
        Commands::Status { verbose } => {
            info!("Showing status{}", if verbose { " (verbose)" } else { "" });
            eprintln!("Showing status{}", if verbose { " (verbose)" } else { "" });

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

            // Get the dotfiles repository path
            let config_path = Config::find_config_file()
                .with_context(|| "No configuration file found. Run 'ordinator init' first.")?
                .ok_or_else(|| {
                    anyhow::anyhow!("No configuration file found. Run 'ordinator init' first.")
                })?;
            let dotfiles_path = config_path.parent().unwrap().to_path_buf();

            let git_manager = GitManager::new(dotfiles_path);
            if !git_manager.exists() {
                return Err(anyhow::anyhow!(
                    "No Git repository found. Run 'ordinator init' first."
                ));
            }

            let status = git_manager.status()?;
            eprintln!("{status}");
            return Ok(());
        }
        Commands::Apply {
            profile,
            skip_bootstrap,
            skip_secrets,
        } => {
            info!("Applying profile: {}", profile);
            eprintln!("Applying profile: {profile}");
            if skip_bootstrap {
                info!("Skipping bootstrap");
                eprintln!("Skipping bootstrap");
            }
            if skip_secrets {
                info!("Skipping secrets");
                eprintln!("Skipping secrets");
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
                return Ok(());
            }

            // Load config and get profile
            let config_path = Config::find_config_file()
                .with_context(|| "No configuration file found. Run 'ordinator init' first.")?
                .ok_or_else(|| {
                    anyhow::anyhow!("No configuration file found. Run 'ordinator init' first.")
                })?;
            let config = Config::from_file(&config_path)?;
            let profile_cfg = config.get_profile(&profile)
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found in config", profile))?;
            let create_backups = config.global.create_backups;

            // For each tracked file, symlink with backup if needed
            use crate::utils::{backup_file_to_dotfiles_backup, is_symlink, get_home_dir};
            let home_dir = get_home_dir()?;
            let dotfiles_dir = config_path.parent().unwrap();
            for file in &profile_cfg.files {
                // Source: dotfiles repo (files/<file>), Dest: home dir/<file>
                let source = dotfiles_dir.join("files").join(file);
                let dest = home_dir.join(file);
                if dest.exists() {
                    // If already correct symlink, skip
                    if is_symlink(&dest) {
                        if let Ok(target) = std::fs::read_link(&dest) {
                            if target == source {
                                eprintln!("Already symlinked: {} -> {}", dest.display(), source.display());
                                continue;
                            }
                        }
                    }
                    // Backup if enabled
                    if create_backups {
                        let backup_path = backup_file_to_dotfiles_backup(&dest, &config_path)?;
                        eprintln!("Backed up {} to {}", dest.display(), backup_path.display());
                    }
                    // Remove the old file
                    std::fs::remove_file(&dest)?;
                }
                // Create parent dirs if needed
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                // Symlink
                #[cfg(unix)]
                std::os::unix::fs::symlink(&source, &dest)?;
                #[cfg(windows)]
                std::os::windows::fs::symlink_file(&source, &dest)?;
                eprintln!("Symlinked {} -> {}", dest.display(), source.display());
            }
            info!("Apply completed");
            eprintln!("Apply completed");
            return Ok(());
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
            if let Some(config) = Config::load()? {
                let profiles = config.list_profiles();
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
            } else {
                return Err(anyhow::anyhow!(
                    "No configuration file found. Run 'ordinator init' first."
                ));
            }
            return Ok(());
        }
        Commands::Secrets { subcommand } => {
            match subcommand {
                SecretCommands::Encrypt { file } => {
                    info!("Encrypting file: {}", file);
                    eprintln!("Encrypting file: {file}");

                    if args.dry_run {
                        info!("[DRY RUN] Would encrypt file: {}", file);
                        eprintln!("DRY-RUN: Would encrypt file: {file}");
                        return Ok(());
                    }

                    // TODO: Implement actual encrypt logic
                    info!("Encryption not yet implemented");
                    eprintln!("Encryption not yet implemented");
                    return Ok(());
                }
                SecretCommands::Decrypt { file } => {
                    info!("Decrypting file: {}", file);
                    eprintln!("Decrypting file: {file}");

                    if args.dry_run {
                        info!("[DRY RUN] Would decrypt file: {}", file);
                        eprintln!("DRY-RUN: Would decrypt file: {file}");
                        return Ok(());
                    }

                    // TODO: Implement actual decrypt logic
                    info!("Decryption not yet implemented");
                    eprintln!("Decryption not yet implemented");
                    return Ok(());
                }
                SecretCommands::List { paths_only } => {
                    info!(
                        "Listing encrypted files{}",
                        if paths_only { " (paths only)" } else { "" }
                    );
                    eprintln!(
                        "Listing encrypted files{}",
                        if paths_only { " (paths only)" } else { "" }
                    );

                    if args.dry_run {
                        info!(
                            "[DRY RUN] Would list encrypted files{}",
                            if paths_only { " (paths only)" } else { "" }
                        );
                        eprintln!(
                            "DRY-RUN: Would list encrypted files{}",
                            if paths_only { " (paths only)" } else { "" }
                        );
                        return Ok(());
                    }

                    // TODO: Implement actual list logic
                    info!("Encrypted files listing not yet implemented");
                    eprintln!("Encrypted files listing not yet implemented");
                    return Ok(());
                }
            }
        }
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
            return Ok(());
        }
    }
}
