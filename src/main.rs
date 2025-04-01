mod cli;
mod commands;
mod config;
mod error;
mod git;
mod logger;
mod ui;

use std::process;

use log::{debug, error};

use crate::cli::Commands;
use crate::error::BasecampError;
use crate::ui::UI;

fn main() {
    // Parse command-line arguments
    let args = cli::parse_args();

    // Initialize logger
    logger::init(args.verbose);

    debug!("Starting BaseCamp");

    // Execute the requested command
    let result = match &args.command {
        Commands::Init => commands::init(),
        Commands::Install { codebase, parallel } => {
            commands::install(codebase.clone(), *parallel)
        }
        Commands::List { codebase } => commands::list(codebase.clone()),
        Commands::Add {
            codebase,
            repositories,
        } => commands::add(codebase.clone(), repositories.clone()),
        Commands::Remove {
            codebase,
            repositories,
            force,
        } => commands::remove(codebase.clone(), repositories.clone(), *force),
    };

    // Handle command result
    if let Err(err) = result {
        handle_error(err);
        process::exit(1);
    }

    debug!("BaseCamp completed successfully");
}

/// Handle application errors
fn handle_error(err: BasecampError) {
    match err {
        BasecampError::GitHubUrlNotConfigured => {
            UI::error("GitHub URL not configured. Run 'basecamp init' first.");
            error!("GitHub URL not configured");
        }
        BasecampError::UncommittedChanges(path) => {
            UI::error(&format!(
                "Repository '{}' has uncommitted changes. Commit or stash your changes, or use --force to override.",
                path.display()
            ));
            error!("Uncommitted changes detected in {}", path.display());
        }
        BasecampError::UnpushedCommits(path) => {
            UI::error(&format!(
                "Repository '{}' has unpushed commits. Push your commits, or use --force to override.",
                path.display()
            ));
            error!("Unpushed commits detected in {}", path.display());
        }
        BasecampError::FileNotFound(path) => {
            UI::error(&format!(
                "File not found: {}. Run 'basecamp init' to create a new configuration.",
                path.display()
            ));
            error!("File not found: {}", path.display());
        }
        BasecampError::CodebaseNotFound(name) => {
            UI::error(&format!("Codebase '{}' not found", name));
            error!("Codebase not found: {}", name);
        }
        BasecampError::RepositoryNotFound(repo, codebase) => {
            UI::error(&format!(
                "Repository '{}' not found in codebase '{}'",
                repo, codebase
            ));
            error!("Repository not found: {} in {}", repo, codebase);
        }
        BasecampError::InvalidGitHubUrl(url) => {
            UI::error(&format!(
                "Invalid GitHub URL: {}. It should start with 'https://' or 'git@'.",
                url
            ));
            error!("Invalid GitHub URL: {}", url);
        }
        _ => {
            UI::error(&format!("Error: {}", err));
            error!("{}", err);
        }
    }
}
