use log::{debug, info};

use crate::config::Config;
use crate::error::{BasecampError, BasecampResult};
use crate::ui::UI;

/// Execute the list command
pub fn execute(codebase: Option<String>) -> BasecampResult<()> {
    debug!("Executing list command");

    // Load configuration
    let config = Config::load(&std::path::PathBuf::new())?;

    // Check if GitHub URL is configured
    if !config.has_github_url() {
        return Err(BasecampError::GitHubUrlNotConfigured);
    }

    // List specific codebase or all codebases
    match codebase {
        Some(codebase_name) => list_repositories(&config, &codebase_name),
        None => list_codebases(&config),
    }
}

/// List all codebases
fn list_codebases(config: &Config) -> BasecampResult<()> {
    info!("Listing all codebases");

    let codebases = config.list_codebases();

    if codebases.is_empty() {
        UI::info("No codebases configured yet. Use 'basecamp add <codebase> <repo>' to add one.");
        return Ok(());
    }

    let mut table = UI::create_table(vec!["Codebase", "Repositories"]);

    for codebase_name in codebases {
        let repos = config.get_repositories(codebase_name)?;
        
        // Format repository names as a simple comma-separated list
        let repo_names = if !repos.is_empty() {
            repos.join(", ")
        } else {
            String::from("None")
        };

        UI::add_table_row(
            &mut table,
            vec![
                codebase_name.to_string(),
                repo_names
            ],
        );
    }

    UI::print_table(&table);

    Ok(())
}

/// List repositories in a specific codebase
fn list_repositories(config: &Config, codebase: &str) -> BasecampResult<()> {
    info!("Listing repositories for codebase: {}", codebase);

    let repos = config.get_repositories(codebase)?;

    if repos.is_empty() {
        UI::info(&format!(
            "No repositories in codebase '{}'. Use 'basecamp add {} <repo>' to add one.",
            codebase, codebase
        ));
        return Ok(());
    }

    let mut table = UI::create_table(vec!["Repository", "URL"]);

    for repo in repos {
        let url = format!("{}/{}.git", config.git_config.github_url, repo);

        UI::add_table_row(&mut table, vec![repo.to_string(), url]);
    }

    UI::print_table(&table);

    Ok(())
}
