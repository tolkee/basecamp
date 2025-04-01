use std::collections::HashMap;
use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

use log::{debug, info};
use serde::{Deserialize, Serialize};

use crate::error::{BasecampError, BasecampResult};

/// Git configuration structure
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct GitConfig {
    /// Base GitHub URL for repositories
    #[serde(default)]
    pub github_url: String,
}

/// Codebases configuration structure
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct CodebasesConfig {
    /// Map of codebase names to repository lists
    #[serde(default)]
    pub codebases: HashMap<String, Vec<String>>,
}

/// Configuration structure for BaseCamp
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Git configuration
    pub git_config: GitConfig,
    /// Codebases configuration
    pub codebases_config: CodebasesConfig,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get path to .basecamp directory
    pub fn get_basecamp_dir() -> PathBuf {
        PathBuf::from(".basecamp")
    }

    /// Get path to config.yaml file
    pub fn get_config_path() -> PathBuf {
        Self::get_basecamp_dir().join("config.yaml")
    }

    /// Get path to codebases.yaml file
    pub fn get_codebases_path() -> PathBuf {
        Self::get_basecamp_dir().join("codebases.yaml")
    }

    /// Ensure the .basecamp directory exists
    pub fn ensure_basecamp_dir() -> BasecampResult<()> {
        let dir = Self::get_basecamp_dir();
        if !dir.exists() {
            debug!("Creating .basecamp directory at {:?}", dir);
            create_dir_all(&dir)?;
        }
        Ok(())
    }

    /// Load configuration from the .basecamp directory files
    pub fn load(_: &Path) -> BasecampResult<Self> {
        // Try to load from the configuration files
        debug!("Loading configuration from .basecamp directory");
        
        // Load git config
        let git_config = if Self::get_config_path().exists() {
            let content = fs::read_to_string(Self::get_config_path())?;
            serde_yaml::from_str(&content)?
        } else {
            return Err(BasecampError::FileNotFound(Self::get_config_path()));
        };
        
        // Load codebases config
        let codebases_config = if Self::get_codebases_path().exists() {
            let content = fs::read_to_string(Self::get_codebases_path())?;
            serde_yaml::from_str(&content)?
        } else {
            CodebasesConfig::default()
        };
        
        let config = Self {
            git_config,
            codebases_config,
        };
        
        info!("Configuration loaded successfully");
        Ok(config)
    }

    /// Save configuration to the .basecamp directory files
    pub fn save(&self, _: &Path) -> BasecampResult<()> {
        // Ensure the directory exists
        Self::ensure_basecamp_dir()?;
        
        // Save each file independently
        let config_result = self.save_config();
        let codebases_result = self.save_codebases();
        
        // Return any error that occurred
        config_result?;
        codebases_result?;
        
        // Verify files exist after saving
        if !Self::get_config_path().exists() {
            return Err(BasecampError::FileNotFound(Self::get_config_path()));
        }
        
        if !Self::get_codebases_path().exists() {
            return Err(BasecampError::FileNotFound(Self::get_codebases_path()));
        }
        
        Ok(())
    }
    
    /// Save git configuration to config.yaml
    pub fn save_config(&self) -> BasecampResult<()> {
        Self::ensure_basecamp_dir()?;
        let config_path = Self::get_config_path();
        debug!("Saving git configuration to {:?}", config_path);
        
        let yaml = serde_yaml::to_string(&self.git_config)?;
        let mut file = File::create(config_path)?;
        file.write_all(yaml.as_bytes())?;
        
        info!("Git configuration saved successfully");
        Ok(())
    }
    
    /// Save codebases configuration to codebases.yaml
    pub fn save_codebases(&self) -> BasecampResult<()> {
        Self::ensure_basecamp_dir()?;
        let codebases_path = Self::get_codebases_path();
        debug!("Saving codebases configuration to {:?}", codebases_path);
        
        let yaml = serde_yaml::to_string(&self.codebases_config)?;
        let mut file = File::create(codebases_path)?;
        file.write_all(yaml.as_bytes())?;
        
        info!("Codebases configuration saved successfully");
        Ok(())
    }

    /// Check if GitHub URL is configured
    pub fn has_github_url(&self) -> bool {
        !self.git_config.github_url.is_empty()
    }

    /// Set GitHub URL
    pub fn set_github_url(&mut self, url: String) -> BasecampResult<()> {
        // Simple validation - could be more sophisticated
        if !url.starts_with("https://") && !url.starts_with("git@") {
            return Err(BasecampError::InvalidGitHubUrl(url));
        }

        self.git_config.github_url = url;
        Ok(())
    }

    /// Remove a codebase
    pub fn remove_codebase(&mut self, name: &str) -> BasecampResult<()> {
        if !self.codebases_config.codebases.contains_key(name) {
            return Err(BasecampError::CodebaseNotFound(name.to_string()));
        }

        self.codebases_config.codebases.remove(name);
        Ok(())
    }

    /// Add repositories to a codebase
    pub fn add_repositories(&mut self, codebase: &str, repos: &[String]) -> BasecampResult<Vec<String>> {
        let codebase_repos = self.codebases_config.codebases.entry(codebase.to_string()).or_default();
        let mut added_repos = Vec::new();
        let mut skipped_repos = Vec::new();

        for repo in repos {
            if codebase_repos.contains(&repo.to_string()) {
                // Skip repos that already exist instead of returning an error
                skipped_repos.push(repo.to_string());
            } else {
                codebase_repos.push(repo.to_string());
                added_repos.push(repo.to_string());
            }
        }

        // Return the list of repos that were actually added (not skipped)
        Ok(added_repos)
    }

    /// Remove repositories from a codebase
    pub fn remove_repositories(&mut self, codebase: &str, repos: &[String]) -> BasecampResult<()> {
        let codebase_repos = match self.codebases_config.codebases.get_mut(codebase) {
            Some(r) => r,
            None => return Err(BasecampError::CodebaseNotFound(codebase.to_string())),
        };

        for repo in repos {
            if !codebase_repos.contains(&repo.to_string()) {
                return Err(BasecampError::RepositoryNotFound(
                    repo.to_string(),
                    codebase.to_string(),
                ));
            }

            codebase_repos.retain(|r| r != repo);
        }

        Ok(())
    }

    /// Get all repositories for a specific codebase
    pub fn get_repositories(&self, codebase: &str) -> BasecampResult<&Vec<String>> {
        match self.codebases_config.codebases.get(codebase) {
            Some(repos) => Ok(repos),
            None => Err(BasecampError::CodebaseNotFound(codebase.to_string())),
        }
    }

    /// List all codebases
    pub fn list_codebases(&self) -> Vec<&String> {
        self.codebases_config.codebases.keys().collect()
    }
}
