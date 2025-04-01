use std::path::PathBuf;
use thiserror::Error;

/// Custom error types for the BaseCamp application
#[derive(Error, Debug)]
pub enum BasecampError {
    #[error("Git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("YAML serialization/deserialization error: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Repository '{0}' not found in codebase '{1}'")]
    RepositoryNotFound(String, String),

    #[error("Codebase '{0}' not found")]
    CodebaseNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Repository at '{0}' has uncommitted changes")]
    UncommittedChanges(PathBuf),

    #[error("Repository at '{0}' has unpushed commits")]
    UnpushedCommits(PathBuf),

    #[error("GitHub URL not configured")]
    GitHubUrlNotConfigured,

    #[error("Invalid GitHub URL: {0}")]
    InvalidGitHubUrl(String),

    #[error("Command failed: {0}")]
    CommandFailed(String),

    #[error("{0}")]
    Generic(String),
}

/// Result type for BaseCamp operations
pub type BasecampResult<T> = std::result::Result<T, BasecampError>;
