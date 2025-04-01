use log::{debug, info};
use std::path::PathBuf;

use crate::config::Config;
use crate::error::{BasecampError, BasecampResult};
use crate::git::GitRepo;
use crate::ui::UI;

/// Execute the remove command
pub fn execute(
    codebase: String,
    repositories: Vec<String>,
    force: bool,
) -> BasecampResult<()> {
    debug!(
        "Executing remove command for codebase '{}' with repos: {:?}",
        codebase, repositories
    );

    // Load configuration
    let mut config = Config::load(&PathBuf::new())?;

    // Check if GitHub URL is configured
    if !config.has_github_url() {
        return Err(BasecampError::GitHubUrlNotConfigured);
    }

    // If no repositories specified, remove the entire codebase
    if repositories.is_empty() {
        return remove_codebase(&mut config, &codebase, force);
    }

    // Otherwise, remove specific repositories
    remove_repositories(&mut config, &codebase, &repositories, force)
}

/// Remove an entire codebase
fn remove_codebase(
    config: &mut Config,
    codebase: &str,
    force: bool,
) -> BasecampResult<()> {
    info!("Removing entire codebase: {}", codebase);

    // Get repositories in the codebase
    let repos = match config.get_repositories(codebase) {
        Ok(r) => r.clone(),
        Err(e) => return Err(e),
    };

    // Check if repositories exist on disk
    let codebase_path = PathBuf::from(codebase);
    let codebase_exists_on_disk = codebase_path.exists();
    
    if codebase_exists_on_disk {
        // Check if force is required
        if !force {
            for repo in &repos {
                let repo_path = GitRepo::get_repo_path(codebase, repo);

                // Check for uncommitted changes
                if repo_path.exists() && GitRepo::has_uncommitted_changes(&repo_path)? {
                    return Err(BasecampError::UncommittedChanges(repo_path));
                }

                // Check for unpushed commits
                if repo_path.exists() && GitRepo::has_unpushed_commits(&repo_path)? {
                    return Err(BasecampError::UnpushedCommits(repo_path));
                }
            }
        }

        // Ask for confirmation
        let confirm = UI::confirm(
            &format!(
                "This will remove codebase '{}' and all of its repositories from the configuration\n\
                 AND DELETE ALL LOCAL FILES in the '{}' directory. Continue?",
                codebase, codebase
            ),
            false,
        )?;

        if !confirm {
            UI::info("Remove cancelled.");
            return Ok(());
        }
    } else {
        // If the codebase doesn't exist on disk, just confirm removal from config
        let confirm = UI::confirm(
            &format!(
                "This will remove codebase '{}' and all of its repositories from the configuration. Continue?",
                codebase
            ),
            false,
        )?;

        if !confirm {
            UI::info("Remove cancelled.");
            return Ok(());
        }
    }

    // Remove codebase from configuration
    config.remove_codebase(codebase)?;

    // Save the updated configuration
    config.save(&PathBuf::new())?;

    UI::success(&format!("Removed codebase '{}' from configuration", codebase));

    // Delete local files if they exist
    if codebase_exists_on_disk {
        UI::info(&format!("Deleting local directory '{}'...", codebase));
        match std::fs::remove_dir_all(&codebase_path) {
            Ok(_) => {
                UI::success(&format!("Successfully deleted local directory '{}'", codebase));
                info!("Deleted local directory '{}'", codebase);
            },
            Err(e) => {
                UI::warning(&format!("Failed to delete local directory '{}': {}", codebase, e));
                info!("Failed to delete local directory '{}': {}", codebase, e);
            }
        }
    }

    Ok(())
}

/// Remove specific repositories from a codebase
fn remove_repositories(
    config: &mut Config,
    codebase: &str,
    repositories: &[String],
    force: bool,
) -> BasecampResult<()> {
    info!(
        "Removing repositories {:?} from codebase '{}'",
        repositories, codebase
    );

    // Track which repositories exist on disk
    let mut repos_on_disk = Vec::new();
    
    // Check if force is required and collect repositories that exist on disk
    if !force {
        for repo in repositories {
            let repo_path = GitRepo::get_repo_path(codebase, repo);
            
            if repo_path.exists() {
                repos_on_disk.push((repo, repo_path.clone()));
                
                // Check for uncommitted changes
                if GitRepo::has_uncommitted_changes(&repo_path)? {
                    return Err(BasecampError::UncommittedChanges(repo_path));
                }

                // Check for unpushed commits
                if GitRepo::has_unpushed_commits(&repo_path)? {
                    return Err(BasecampError::UnpushedCommits(repo_path));
                }
            }
        }
    } else {
        // If force is enabled, just collect repositories that exist on disk
        for repo in repositories {
            let repo_path = GitRepo::get_repo_path(codebase, repo);
            if repo_path.exists() {
                repos_on_disk.push((repo, repo_path.clone()));
            }
        }
    }
    
    // Create confirmation message based on whether repos exist on disk
    let confirmation_message = if !repos_on_disk.is_empty() {
        format!(
            "This will remove repositories {:?} from codebase '{}'\n\
             AND DELETE THE FOLLOWING LOCAL DIRECTORIES:\n{}\n\
             Continue?",
            repositories, codebase,
            repos_on_disk.iter().map(|(_, path)| format!("  - {}", path.display())).collect::<Vec<_>>().join("\n")
        )
    } else {
        format!(
            "This will remove repositories {:?} from codebase '{}' configuration. Continue?",
            repositories, codebase
        )
    };

    // Ask for confirmation
    let confirm = UI::confirm(&confirmation_message, false)?;

    if !confirm {
        UI::info("Remove cancelled.");
        return Ok(());
    }

    // Remove repositories from codebase configuration
    config.remove_repositories(codebase, repositories)?;

    // Save the updated configuration
    config.save(&PathBuf::new())?;

    let repo_list = repositories.join(", ");
    UI::success(&format!(
        "Removed repositories [{}] from codebase '{}' configuration",
        repo_list, codebase
    ));
    
    // Delete local files for each repository
    if !repos_on_disk.is_empty() {
        UI::info("Deleting local repository directories...");
        
        for (repo, repo_path) in repos_on_disk {
            match std::fs::remove_dir_all(&repo_path) {
                Ok(_) => {
                    UI::success(&format!("Successfully deleted local directory for '{}'", repo));
                    info!("Deleted local directory '{}'", repo_path.display());
                },
                Err(e) => {
                    UI::warning(&format!("Failed to delete local directory for '{}': {}", repo, e));
                    info!("Failed to delete local directory '{}': {}", repo_path.display(), e);
                }
            }
        }
    }

    Ok(())
}
