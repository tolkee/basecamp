use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a temporary directory and return its path
pub fn setup_temp_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path().to_path_buf();
    (temp_dir, temp_path)
}

/// Create a test configuration file with a sample GitHub URL
pub fn create_test_config(base_path: &Path) -> PathBuf {
    // Create .basecamp directory
    let basecamp_dir = base_path.join(".basecamp");
    fs::create_dir_all(&basecamp_dir).expect("Failed to create .basecamp directory");
    
    // Create config.yaml for git configuration
    let config_path = basecamp_dir.join("config.yaml");
    let config_content = r#"github_url: https://github.com/test-org"#;
    fs::write(&config_path, config_content).expect("Failed to write config.yaml file");
    
    // Create codebases.yaml for codebase configuration
    let codebases_path = basecamp_dir.join("codebases.yaml");
    let codebases_content = r#"codebases:
  frontend:
    - ui-component
    - web-client
  backend:
    - api-server
    - database"#;
    fs::write(&codebases_path, codebases_content).expect("Failed to write codebases.yaml file");
    
    // Return the basecamp_dir path
    basecamp_dir
}

/// Remove a temporary directory and its contents
pub fn teardown(temp_dir: TempDir) {
    temp_dir
        .close()
        .expect("Failed to remove temporary directory");
}
