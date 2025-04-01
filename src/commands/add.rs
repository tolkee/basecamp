use log::{debug, info};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::Config;
use crate::error::{BasecampError, BasecampResult};
use crate::ui::UI;
use crate::git::GitRepo;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

/// Execute the add command
pub fn execute(
    codebase: String,
    repositories: Vec<String>,
) -> BasecampResult<()> {
    debug!(
        "Executing add command for codebase '{}' with repos: {:?}",
        codebase, repositories
    );

    if repositories.is_empty() {
        return Err(BasecampError::Generic(
            "No repositories specified".to_string(),
        ));
    }

    // Load configuration
    let mut config = match Config::load(&PathBuf::new()) {
        Ok(config) => config,
        Err(BasecampError::FileNotFound(_)) => {
            // If config file doesn't exist, ask for GitHub URL
            UI::info("No configuration file found. Creating a new one.");
            UI::info("Please enter your GitHub URL:");
            UI::info("Examples:");
            UI::info("  - https://github.com/your-org");
            UI::info("  - git@github.com:your-org");

            let url: String = UI::input("GitHub URL", None)?;

            let mut new_config = Config::new();
            new_config.set_github_url(url)?;
            new_config
        }
        Err(e) => return Err(e),
    };

    // Check if GitHub URL is configured
    if !config.has_github_url() {
        return Err(BasecampError::GitHubUrlNotConfigured);
    }

    // Add repositories to codebase
    match config.add_repositories(&codebase, &repositories) {
        Ok(added_repos) => {
            // Save the updated configuration
            config.save(&PathBuf::new())?;

            // Determine which repos were skipped (those in repositories but not in added_repos)
            let skipped_repos: Vec<String> = repositories.iter()
                .filter(|repo| !added_repos.contains(&repo.to_string()))
                .map(|repo| repo.to_string())
                .collect();
            
            if !skipped_repos.is_empty() {
                let skipped_list = skipped_repos.join(", ");
                UI::info(&format!(
                    "Skipped repositories that already exist [{}] in codebase '{}'",
                    skipped_list, codebase
                ));
            }
            
            if !added_repos.is_empty() {
                let added_list = added_repos.join(", ");
                UI::success(&format!(
                    "Added repositories [{}] to codebase '{}'",
                    added_list, codebase
                ));
                info!("Added repositories to codebase '{}'", codebase);

                // Install the newly added repositories
                UI::info(&format!("Installing {} new repositories...", added_repos.len()));
                
                // Default to 4 parallel installations (same as default in CLI)
                let parallel_count = 4;
                
                // Install only the new repositories
                match install_new_repositories(&config, &codebase, &added_repos, parallel_count) {
                    Ok(_) => {
                        UI::success(&format!("Successfully installed new repositories for codebase '{}'", codebase));
                    }
                    Err(e) => {
                        UI::warning(&format!("Installation failed: {}", e));
                        
                        // Get the list of failed repositories from the error
                        let failed_repos = if let BasecampError::CommandFailed(_) = &e {
                            let failed_repos_list = get_failed_repositories(&e);
                            if !failed_repos_list.is_empty() {
                                Some(failed_repos_list)
                            } else {
                                None
                            }
                        } else {
                            // If it's another type of error, assume all new repositories failed
                            Some(added_repos.clone())
                        };
                        
                        // If we have failed repositories, remove them from config
                        if let Some(repos_to_remove) = failed_repos {
                            // Format the list of repositories to remove for display
                            let repos_to_remove_str = repos_to_remove.join(", ");
                            UI::info(&format!("Removing failed repositories [{}] from configuration...", repos_to_remove_str));
                            
                            // Load a fresh copy of the config to avoid conflicts
                            match Config::load(&PathBuf::new()) {
                                Ok(mut updated_config) => {
                                    let rollback_result = updated_config.remove_repositories(&codebase, &repos_to_remove);
                                    
                                    if let Ok(_) = rollback_result {
                                        // Save the updated configuration without the failed repos
                                        if let Ok(_) = updated_config.save(&PathBuf::new()) {
                                            UI::success(&format!(
                                                "Removed failed repositories [{}] from codebase '{}'",
                                                repos_to_remove_str, codebase
                                            ));
                                        } else {
                                            UI::error(&format!(
                                                "Failed to save updated configuration after removing failed repositories [{}]",
                                                repos_to_remove_str
                                            ));
                                        }
                                    } else {
                                        UI::error(&format!(
                                            "Failed to remove repositories [{}] from configuration",
                                            repos_to_remove_str
                                        ));
                                    }
                                }
                                Err(_) => {
                                    UI::error("Failed to reload configuration for cleanup");
                                }
                            }
                        }
                    }
                }
            } else {
                UI::info("No new repositories to install.");
            }

            Ok(())
        }
        Err(e) => {
            UI::error(&format!("Failed to add repositories: {}", e));
            Err(e)
        }
    }
}

/// Extract failed repository names from an error
fn get_failed_repositories(error: &BasecampError) -> Vec<String> {
    if let BasecampError::CommandFailed(msg) = error {
        // In install_new_repositories, we format the error message with the list of failed repos
        // Format is "{count} repositories failed to clone: {comma_separated_list}"
        if let Some(repo_list_part) = msg.split(": ").nth(1) {
            // Split the comma-separated list and collect repo names
            return repo_list_part.split(", ")
                .map(|s| s.trim().to_string())
                .collect();
        }
    }
    
    // If we couldn't extract specific repositories, return an empty list
    Vec::new()
}

/// Install only specific repositories in a codebase
fn install_new_repositories(
    config: &Config, 
    codebase: &str, 
    repositories: &[String], 
    parallel_count: usize
) -> BasecampResult<()> {
    if repositories.is_empty() {
        return Ok(());
    }

    let total_repos = repositories.len();

    // Display what will be installed
    UI::info(&format!(
        "Installing {} new repositories in codebase '{}'",
        total_repos, codebase
    ));

    // Adjust parallel count based on available repositories
    let parallel_count = std::cmp::min(parallel_count, total_repos);

    // Create shared data for threads
    let multi_progress = Arc::new(MultiProgress::new());
    let repos_to_install = Arc::new(repositories.to_vec());
    let error_repos = Arc::new(Mutex::new(Vec::new()));
    let parallel_count = std::cmp::min(parallel_count, repos_to_install.len());
    let github_url = config.git_config.github_url.clone();
    let codebase = Arc::new(codebase.to_string());
    let remaining_repos = Arc::new(Mutex::new((0..total_repos).collect::<Vec<_>>()));
    let completed_repos = Arc::new(Mutex::new(0));
    
    // Setup progress bars
    let multi_progress_arc = multi_progress.clone();
    
    // Create the main progress bar
    let progress_bar = multi_progress_arc.add(ProgressBar::new(total_repos as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .expect("Failed to create progress bar template")
            .progress_chars("=> ")
    );
    progress_bar.set_message(format!("Installing new repositories in '{}'", codebase));
    
    // Spinner style for individual repositories
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {wide_msg}")
        .expect("Failed to create spinner style template");

    // Create a clone of MultiProgress for the worker threads
    let mp_for_threads = multi_progress_arc.clone();
    
    // Spawn worker threads
    let mut handles = vec![];

    for _ in 0..parallel_count {
        let repos = Arc::clone(&repos_to_install);
        let codebase = Arc::clone(&codebase);
        let remaining_repos = Arc::clone(&remaining_repos);
        let errors = Arc::clone(&error_repos);
        let github_url = github_url.clone();
        let multi_progress = Arc::clone(&mp_for_threads);
        let spinner_style = spinner_style.clone();
        let completed_repos = Arc::clone(&completed_repos);
        let progress_bar = progress_bar.clone();

        let handle = thread::spawn(move || {
            loop {
                // Get next repository to clone
                let repo_idx = {
                    let mut remaining = remaining_repos.lock().unwrap();
                    if remaining.is_empty() {
                        break;
                    }
                    remaining.remove(0)
                };

                let repo = &repos[repo_idx];
                
                // Create a new spinner for this repository
                let spinner = multi_progress.add(ProgressBar::new_spinner());
                spinner.set_style(spinner_style.clone());
                spinner.set_message(format!("Cloning '{}'...", repo));
                spinner.enable_steady_tick(std::time::Duration::from_millis(100));
                
                // Clone repository
                let repo_path = GitRepo::get_repo_path(&codebase, repo);

                if repo_path.exists() {
                    spinner.set_message(format!("Repository '{}' already exists, skipping", repo));
                    spinner.finish_with_message(format!("Repository '{}' already exists, skipped ✓", repo));
                    // Not an error - just a skip
                } else {
                    let repo_url = GitRepo::build_repo_url(&github_url, repo);

                    match GitRepo::clone(&repo_url, &repo_path) {
                        Ok(_) => {
                            spinner.finish_with_message(format!("Cloned '{}' successfully ✓", repo));
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to clone repository '{}': {}", repo, e);
                            spinner.finish_with_message(format!("Failed to clone '{}' ✗", repo));

                            // Add error to the list
                            let mut errors_list = errors.lock().unwrap();
                            errors_list.push((repo.clone(), error_msg));
                        }
                    }
                }
                
                // Update progress
                {
                    let mut completed = completed_repos.lock().unwrap();
                    *completed += 1;
                    progress_bar.set_position(*completed as u64);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        let _ = handle.join();
    }
    
    // Check for errors before finishing the progress bar
    let errors_list = error_repos.lock().unwrap();
    if !errors_list.is_empty() {
        // Change progress bar to indicate errors
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.red/blue}] {pos}/{len} ({percent}%)")
                .expect("Failed to create progress bar template")
                .progress_chars("=> ")
        );
        progress_bar.finish_with_message(format!("Installation of repositories in '{}' completed with errors", codebase));

        UI::warning(&format!(
            "Encountered {} errors during installation:",
            errors_list.len()
        ));

        // Create a list of failed repository names
        let failed_repos: Vec<String> = errors_list.iter()
            .map(|(repo, _)| repo.clone())
            .collect();
        
        for (repo, error) in errors_list.iter() {
            UI::error(&format!("  {}: {}", repo, error));
        }

        return Err(BasecampError::CommandFailed(format!(
            "{} repositories failed to clone: {}",
            errors_list.len(),
            failed_repos.join(", ")
        )));
    } else {
        // All went well, finish with a success message
        progress_bar.finish_with_message(format!("Successfully completed installing new repositories in '{}'", codebase));
    }

    // Let Arc<MultiProgress> clean up naturally when all references are dropped
    // The worker threads have all completed, so their references are gone
    // This is the last reference, ensuring proper cleanup
    drop(multi_progress_arc);

    Ok(())
}
