use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use log::{debug, info};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

use crate::config::Config;
use crate::error::{BasecampError, BasecampResult};
use crate::git::GitRepo;
use crate::ui::UI;

/// Execute the install command
pub fn execute(
    codebase: Option<String>,
    parallel_count: usize,
) -> BasecampResult<()> {
    debug!("Executing install command");

    // Load configuration
    let config = Config::load(&PathBuf::new())?;

    // Check if GitHub URL is configured
    if !config.has_github_url() {
        return Err(BasecampError::GitHubUrlNotConfigured);
    }

    // Install specific codebase or all codebases
    match codebase {
        Some(codebase_name) => install_codebase(&config, &codebase_name, parallel_count),
        None => install_all_codebases(&config, parallel_count),
    }
}

/// Install a specific codebase
fn install_codebase(config: &Config, codebase: &str, parallel_count: usize) -> BasecampResult<()> {
    info!("Installing codebase: {}", codebase);

    // Get repositories for the codebase
    let repos = config.get_repositories(codebase)?;

    if repos.is_empty() {
        UI::info(&format!("No repositories in codebase '{}'", codebase));
        return Ok(());
    }

    // Clone repositories
    clone_repositories(config, codebase, repos, parallel_count)
}

/// Install all codebases
fn install_all_codebases(config: &Config, parallel_count: usize) -> BasecampResult<()> {
    info!("Installing all codebases");

    let codebases = config.list_codebases();

    if codebases.is_empty() {
        UI::info("No codebases configured yet. Use 'basecamp add <codebase> <repo>' to add one.");
        return Ok(());
    }

    // Install each codebase
    for codebase in codebases {
        UI::info(&format!("Installing codebase: {}", codebase));

        let repos = config.get_repositories(codebase)?;

        if repos.is_empty() {
            UI::info(&format!("No repositories in codebase '{}'", codebase));
            continue;
        }

        // Clone repositories
        clone_repositories(config, codebase, repos, parallel_count)?;
    }

    Ok(())
}

/// Clone repositories in parallel
fn clone_repositories(
    config: &Config,
    codebase: &str,
    repos: &[String],
    parallel_count: usize,
) -> BasecampResult<()> {
    if repos.is_empty() {
        return Ok(());
    }

    let total_repos = repos.len();

    // Display what will be installed
    UI::info(&format!(
        "Installing {} repositories in codebase '{}'",
        total_repos, codebase
    ));

    // Adjust parallel count based on available repositories
    let parallel_count = std::cmp::min(parallel_count, total_repos);

    // Create shared data for threads
    let github_url = config.git_config.github_url.clone();
    let repos = Arc::new(repos.to_vec());
    let codebase = Arc::new(codebase.to_string());
    let remaining_repos = Arc::new(Mutex::new((0..total_repos).collect::<Vec<_>>()));
    let errors = Arc::new(Mutex::new(Vec::new()));
    
    // Track completed repositories
    let completed_repos = Arc::new(Mutex::new(0));
    
    // Track repositories that were already installed
    let already_installed_repos = Arc::new(Mutex::new(Vec::new()));
    
    // Setup progress bars
    let multi_progress = MultiProgress::new();
    let multi_progress_arc = Arc::new(multi_progress);
    
    // Create the main progress bar
    let progress_bar = multi_progress_arc.add(ProgressBar::new(total_repos as u64));
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
            .expect("Failed to create progress bar template")
            .progress_chars("=> ")
    );
    progress_bar.set_message(format!("Installing repositories in '{}'", codebase));
    
    // Spinner style for individual repositories
    let spinner_style = ProgressStyle::default_spinner()
        .template("{spinner:.green} {wide_msg}")
        .expect("Failed to create spinner style template");

    // Create a clone of MultiProgress for the worker threads
    let mp_for_threads = multi_progress_arc.clone();
    
    // Spawn worker threads
    let mut handles = vec![];

    for _ in 0..parallel_count {
        let repos = Arc::clone(&repos);
        let codebase = Arc::clone(&codebase);
        let remaining_repos = Arc::clone(&remaining_repos);
        let errors = Arc::clone(&errors);
        let already_installed_repos = Arc::clone(&already_installed_repos);
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
                    // Repository already exists - show a clear already installed message
                    spinner.finish_with_message(format!("Repository '{}' already installed ✓", repo));
                    
                    // Track that this repository was already installed
                    let mut installed = already_installed_repos.lock().unwrap();
                    installed.push(repo.clone());
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
    
    // Get the list of repositories that were already installed
    let already_installed = already_installed_repos.lock().unwrap();
    let newly_installed = total_repos - already_installed.len() - errors.lock().unwrap().len();
    
    // Check for errors before finishing the progress bar
    let errors_list = errors.lock().unwrap();
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

        println!(); // Add padding above errors without the "i" prefix
        for (repo, error) in errors_list.iter() {
            UI::error(&format!("  {}: {}", repo, error));
        }
        println!(); // Add padding below errors without the "i" prefix

        return Err(BasecampError::CommandFailed(format!(
            "{} repositories failed to clone",
            errors_list.len()
        )));
    } else if already_installed.len() == total_repos {
        // All repositories were already installed
        progress_bar.finish_with_message(format!("Codebase '{}' is already up to date", codebase));
        UI::success(&format!("Codebase '{}' is already up to date", codebase));
    } else {
        // Some repositories were installed and some were already present
        if newly_installed > 0 {
            progress_bar.finish_with_message(format!("Successfully installed {} new repositories in '{}'", newly_installed, codebase));
            
            if !already_installed.is_empty() {
                UI::info(&format!("{} repositories were already installed", already_installed.len()));
            }
            
            UI::success(&format!("Successfully installed codebase '{}'", codebase));
        } else {
            // This should not happen (would be caught by the already_installed.len() == total_repos check above)
            progress_bar.finish_with_message(format!("No new repositories were installed in '{}'", codebase));
            UI::success(&format!("Codebase '{}' is already up to date", codebase));
        }
    }

    // Let Arc<MultiProgress> clean up naturally when all references are dropped
    // The worker threads have all completed, so their references are gone
    // This is the last reference, ensuring proper cleanup
    drop(multi_progress_arc);

    Ok(())
}
