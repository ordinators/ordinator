use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
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
        Commands::Init { remote, profile } => {
            info!("Initializing repository with profile: {}", profile);
            if !args.quiet {
                eprintln!("Initializing repository with profile: {profile}");
            }
            if let Some(url) = &remote {
                info!("Remote URL: {}", url);
                if !args.quiet {
                    eprintln!("Remote URL: {url}");
                }
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
            Ok(())
        }
        Commands::Add { path, profile } => {
            let (mut config, config_path) = Config::load()?;
            let profile_name = profile.unwrap_or_else(|| config.global.default_profile.clone());
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
            config.add_file_to_profile(&profile_name, path.clone())?;
            config.save_to_file(&config_path)?;
            println!("Added '{path}' to profile '{profile_name}'");
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
        Commands::Commit { message } => {
            info!("Committing with message: {}", message);
            eprintln!("Committing with message: {message}");

            if args.dry_run {
                info!("[DRY RUN] Would commit with message: {}", message);
                eprintln!("DRY-RUN: Would commit with message: {message}");
                return Ok(());
            }

            // Load config and get dotfiles repo path
            let (_config, config_path) = Config::load()?;
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
            Ok(())
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

            // Load config and get dotfiles repo path
            let (_config, config_path) = Config::load()?;
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
            let git_manager = GitManager::new(dotfiles_path);
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
            let git_manager = GitManager::new(dotfiles_path);

            // Show Git status if repository exists
            if git_manager.exists() {
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
            force,
        } => {
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
                return Ok(());
            }

            // Load config and get profile
            let (config, config_path) = Config::load()?;
            let profile_cfg = config
                .get_profile(&profile)
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found in config", profile))?;
            let create_backups = config.global.create_backups;

            // Debug: print config information
            eprintln!("[DEBUG] Config loaded from: {}", config_path.display());
            eprintln!("[DEBUG] Requested profile: '{profile}'");
            eprintln!(
                "[DEBUG] Profile exists: {}",
                config.profiles.contains_key(&profile)
            );
            eprintln!("[DEBUG] Available profiles: {:?}", config.list_profiles());
            eprintln!("[DEBUG] Profile files count: {}", profile_cfg.files.len());
            eprintln!("[DEBUG] Profile files: {:?}", profile_cfg.files);

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
                profile_cfg.files.len()
            );
            for file in &profile_cfg.files {
                eprintln!("[DEBUG]   - {file}");
            }

            for file in &profile_cfg.files {
                // Source: dotfiles repo (files/<file>), Dest: home dir/<file>
                let dest = home_dir.join(file);

                eprintln!("[DEBUG] Checking file: {file}");
                eprintln!("[DEBUG] Dest: {}", dest.display());

                if !dest.exists() {
                    // Create new symlink
                    eprintln!(
                        "[DEBUG] Creating new symlink: {} -> {}",
                        dest.display(),
                        _dotfiles_dir.join("files").join(file).display()
                    );
                    if args.dry_run {
                        eprintln!(
                            "DRY-RUN: Would create symlink {} -> {}",
                            dest.display(),
                            _dotfiles_dir.join("files").join(file).display()
                        );
                    } else {
                        create_symlink_with_conflict_resolution(
                            &_dotfiles_dir.join("files").join(file),
                            &dest,
                            force,
                            create_backups,
                            &config_path,
                        )?;
                        if !args.quiet {
                            eprintln!(
                                "Symlinked: {} -> {}",
                                dest.display(),
                                _dotfiles_dir.join("files").join(file).display()
                            );
                        }
                    }
                    continue;
                }

                if !is_symlink(&dest) {
                    // Handle non-symlink conflict
                    eprintln!("[DEBUG] Non-symlink conflict: {}", dest.display());
                    if !force {
                        eprintln!(
                            "  {}: Already exists and is not a symlink. Use --force to overwrite.",
                            dest.display()
                        );
                        return Err(anyhow::anyhow!(
                            "Target {} already exists and is not a symlink. Use --force to overwrite.",
                            dest.display()
                        ));
                    }
                    // Force overwrite - create symlink
                    eprintln!(
                        "[DEBUG] Force creating symlink: {} -> {}",
                        dest.display(),
                        _dotfiles_dir.join("files").join(file).display()
                    );
                    if args.dry_run {
                        eprintln!(
                            "DRY-RUN: Would force create symlink {} -> {}",
                            dest.display(),
                            _dotfiles_dir.join("files").join(file).display()
                        );
                    } else {
                        create_symlink_with_conflict_resolution(
                            &_dotfiles_dir.join("files").join(file),
                            &dest,
                            force,
                            create_backups,
                            &config_path,
                        )?;
                        if !args.quiet {
                            eprintln!(
                                "Symlinked: {} -> {}",
                                dest.display(),
                                _dotfiles_dir.join("files").join(file).display()
                            );
                        }
                    }
                    continue;
                }

                // Check if existing symlink is broken or points to wrong target
                let needs_repair = match get_symlink_target(&dest) {
                    Ok(actual_target) => {
                        eprintln!("[DEBUG] actual_target: {}", actual_target.display());
                        eprintln!(
                            "[DEBUG] expected source: {}",
                            _dotfiles_dir.join("files").join(file).display()
                        );
                        eprintln!("[DEBUG] actual_target.exists(): {}", actual_target.exists());
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
                    }
                } else if args.verbose {
                    eprintln!("  {}: Valid symlink", dest.display());
                }
            }
            info!("Apply completed");
            if !args.quiet {
                eprintln!("Apply completed");
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
                    Ok(())
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
                    Ok(())
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
            Ok(())
        }
    }
}
