use clap::Parser;
use tracing::{error, info};

mod bootstrap;
mod brew;
mod cli;
mod config;
mod git;
mod repo;
mod secrets;
mod utils;

use cli::Args;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Ordinator...");

    // Parse command line arguments
    let args = Args::parse();

    // Run the application
    if let Err(e) = cli::run(args).await {
        error!("Application error: {}", e);
        eprintln!("{e}"); // Print error to stderr for user visibility
        std::process::exit(1);
    }

    info!("Ordinator completed successfully");
}
