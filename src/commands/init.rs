use log::{debug, info};
use std::path::PathBuf;
use std::env;

use crate::config::Config;
use crate::error::BasecampResult;
use crate::ui::UI;

/// Execute the init command
pub fn execute() -> BasecampResult<()> {
    debug!("Executing init command");

    // Get paths to the configuration files
    let config_path = Config::get_config_path();
    let codebases_path = Config::get_codebases_path();
    
    // Create the .basecamp directory if it doesn't exist
    if let Err(e) = Config::ensure_basecamp_dir() {
        return Err(crate::error::BasecampError::Generic(format!(
            "Failed to create .basecamp directory: {}",
            e
        )));
    }
    
    // Check if configuration files already exist
    let config_exists = config_path.exists();
    let codebases_exists = codebases_path.exists();
    
    // Get current working directory for better messaging
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    
    if config_exists || codebases_exists {
        let confirm = UI::confirm(
            &format!(
                "Configuration files already exist in {}/.basecamp. Overwrite?",
                current_dir.display()
            ),
            false,
        )?;

        if !confirm {
            UI::info("Init cancelled. Existing configuration preserved.");
            return Ok(());
        }
    }

    // Create new configuration
    let mut config = Config::new();

    // Interactive flow to build GitHub URL
    UI::info("Let's set up your GitHub connection:");
    
    // Ask about connection type using arrow key selection
    let connection_options = &["HTTPS (https://github.com/...)", "SSH (git@github.com:...)"];
    let connection_type = UI::select("What type of connection do you want to use?", connection_options, Some(0))?;
    let is_https = connection_type == 0;
    
    // Ask about repository type using arrow key selection
    let repo_options = &["Organization repositories", "Personal repositories"];
    let repo_type = UI::select("Are you connecting to organization or personal repositories?", repo_options, Some(0))?;
    let is_org = repo_type == 0;
    
    // Ask for org name or username
    let prompt = if is_org {
        "Enter your organization name"
    } else {
        "Enter your GitHub username"
    };
    
    let name: String = UI::input(prompt, None)?;
    
    // Build the GitHub URL based on user choices
    let url = if is_https {
        format!("https://github.com/{}", name)
    } else {
        format!("git@github.com:{}", name)
    };
    
    UI::info(&format!("\nYour GitHub URL will be: {}", url));
    
    let confirm = UI::confirm("Is this correct?", true)?;
    if !confirm {
        UI::info("Let's try again.");
        return execute();
    }
    
    config.set_github_url(url)?;

    // Save the configuration (this will save both config.yaml and codebases.yaml)
    config.save_config()?;
    config.save_codebases()?;

    UI::success(&format!(
        "BaseCamp initialized with configuration in {}/.basecamp",
        current_dir.display()
    ));
    info!("BaseCamp initialized successfully");

    Ok(())
}
