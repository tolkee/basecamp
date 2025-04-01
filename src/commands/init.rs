use log::{debug, info};
use std::path::PathBuf;
use std::env;

use crate::config::Config;
use crate::error::{BasecampError, BasecampResult};
use crate::ui::UI;

/// Execute the init command
pub fn execute(
    connection_type: Option<String>, 
    repo_type: Option<String>, 
    name: Option<String>, 
    non_interactive: bool, 
    force: bool
) -> BasecampResult<()> {
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
        if non_interactive {
            // In non-interactive mode, we use the force flag
            if !force {
                UI::info("Init cancelled. Existing configuration preserved (non-interactive mode).");
                return Ok(());
            }
        } else {
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
    }

    // Create new configuration
    let mut config = Config::new();
    
    // If in non-interactive mode, use command-line parameters
    if non_interactive {
        // Build GitHub URL from the individual parameters
        let conn_type = match connection_type.as_deref() {
            Some("https") => true,
            Some("ssh") => false,
            Some(t) => return Err(BasecampError::Generic(format!("Invalid connection type: {}. Use 'https' or 'ssh'", t))),
            None => return Err(BasecampError::Generic("In non-interactive mode, connection-type must be provided".to_string())),
        };
        
        // Validate repo_type but we don't actually use it for URL construction
        match repo_type.as_deref() {
            Some("org") | Some("personal") => (),
            Some(t) => return Err(BasecampError::Generic(format!("Invalid repository type: {}. Use 'org' or 'personal'", t))),
            None => return Err(BasecampError::Generic("In non-interactive mode, repo-type must be provided".to_string())),
        };
        
        let username_or_org = match name {
            Some(n) => n,
            None => return Err(BasecampError::Generic("In non-interactive mode, name must be provided".to_string())),
        };
        
        // Build the GitHub URL based on user parameters
        let url = if conn_type {
            format!("https://github.com/{}", username_or_org)
        } else {
            format!("git@github.com:{}", username_or_org)
        };
        
        config.set_github_url(url)?;
        UI::info(&format!("Using GitHub URL built from parameters: {}", config.git_config.github_url));
    } else {
        // Interactive flow to build GitHub URL
        UI::info("Let's set up your GitHub connection:");
        
        // Ask about connection type using arrow key selection
        let connection_options = &["HTTPS (https://github.com/...)", "SSH (git@github.com:...)"];
        let connection_type_idx = UI::select("What type of connection do you want to use?", connection_options, Some(0))?;
        let is_https = connection_type_idx == 0;
        
        // Ask about repository type using arrow key selection
        let repo_options = &["Organization repositories", "Personal repositories"];
        let repo_type_idx = UI::select("Are you connecting to organization or personal repositories?", repo_options, Some(0))?;
        let is_org = repo_type_idx == 0;
        
        // Ask for org name or username
        let prompt = if is_org {
            "Enter your organization name"
        } else {
            "Enter your GitHub username"
        };
        
        let name_input: String = UI::input(prompt, None)?;
        
        // Build the GitHub URL based on user choices
        let url = if is_https {
            format!("https://github.com/{}", name_input)
        } else {
            format!("git@github.com:{}", name_input)
        };
        
        UI::info(&format!("\nYour GitHub URL will be: {}", url));
        
        let confirm = UI::confirm("Is this correct?", true)?;
        if !confirm {
            UI::info("Let's try again.");
            return execute(None, None, None, false, false);
        }
        
        config.set_github_url(url)?;
    }

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
