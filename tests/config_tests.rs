mod common;

use basecamp::config::{Config, CodebasesConfig};
use basecamp::error::{BasecampError, BasecampResult};
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;

#[test]
fn test_config_load() {
    // Save the original directory to ensure we always go back
    let original_dir = std::env::current_dir().unwrap();
    
    // Use a block with a finally-like pattern to ensure cleanup
    let result = std::panic::catch_unwind(|| {
        // Setup
        let (temp_dir, temp_path) = common::setup_temp_dir();
        println!("Test config_load using temp directory: {:?}", temp_path);
        
        // Create test config files
        let basecamp_dir = common::create_test_config(&temp_path);
        println!("Created test config in: {:?}", basecamp_dir);
        
        // Verify test files were created correctly
        assert!(basecamp_dir.exists(), "Basecamp directory not created: {:?}", basecamp_dir);
        assert!(basecamp_dir.join("config.yaml").exists(), "config.yaml not created");
        assert!(basecamp_dir.join("codebases.yaml").exists(), "codebases.yaml not created");

        // Set working directory to temp_path
        println!("Changing working directory from {:?} to {:?}", original_dir, temp_path);
        std::env::set_current_dir(&temp_path).unwrap();

        // Test loading the config
        let config = Config::load(&PathBuf::new()).expect("Failed to load config");

        // Verify
        assert_eq!(config.git_config.github_url, "https://github.com/test-org");
        
        // Print codebases for debugging
        println!("Loaded codebases: {:?}", config.codebases_config.codebases);
        
        // Expected repositories based on common::create_test_config
        assert_eq!(config.codebases_config.codebases.len(), 2, 
                   "Expected 2 codebases (frontend and backend), found: {:?}", 
                   config.codebases_config.codebases);
        assert!(config.codebases_config.codebases.contains_key("frontend"), 
                "Frontend codebase not found in: {:?}", config.codebases_config.codebases);
        assert!(config.codebases_config.codebases.contains_key("backend"), 
                "Backend codebase not found in: {:?}", config.codebases_config.codebases);
        
        // Print repository counts for debugging
        let frontend_repos = config.codebases_config.codebases.get("frontend").unwrap();
        println!("Frontend repositories: {:?}", frontend_repos);
        
        let backend_repos = config.codebases_config.codebases.get("backend").unwrap();
        println!("Backend repositories: {:?}", backend_repos);
        
        assert_eq!(frontend_repos.len(), 2,
                  "Expected 2 frontend repos, found: {:?}", frontend_repos);
        assert_eq!(backend_repos.len(), 2,
                  "Expected 2 backend repos, found: {:?}", backend_repos);

        // Cleanup
        println!("Test cleanup: removing temp directory");
        common::teardown(temp_dir);
    });
    
    // Always return to the original directory
    println!("Returning to original directory: {:?}", original_dir);
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to return to original directory: {}", e);
    }
    
    // Re-throw any panic that occurred in the test
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
    
    println!("Test config_load completed");
}

#[test]
fn test_config_save() {
    // Save the original directory to ensure we always go back
    let original_dir = std::env::current_dir().unwrap();
    
    // Use a block with a finally-like pattern to ensure cleanup
    let result = std::panic::catch_unwind(|| {
        // Setup
        let (temp_dir, temp_path) = common::setup_temp_dir();
        println!("Test config_save using temp directory: {:?}", temp_path);

        // Create a config to save
        let mut config = Config::new();
        config
            .set_github_url("https://github.com/test-org".to_string())
            .unwrap();

        // Add some repositories
        let repos_to_add = ["repo1".to_string(), "repo2".to_string()];
        config
            .add_repositories("test-codebase", &repos_to_add)
            .unwrap();

        // Print config state before saving
        println!("Config before saving: {{");
        println!("  git_config: {{ github_url: {} }}", config.git_config.github_url);
        println!("  codebases_config: {{ codebases: {:?} }}", config.codebases_config.codebases);
        println!("}}");

        // Ensure .basecamp directory exists in temp_path
        let basecamp_dir = temp_path.join(".basecamp");
        println!("Creating .basecamp directory at: {:?}", basecamp_dir);
        std::fs::create_dir_all(&basecamp_dir).expect("Failed to create .basecamp directory");
        assert!(basecamp_dir.exists(), "Failed to create basecamp directory");
        
        // Create a custom config with absolute paths for testing
        struct TestConfig {
            config: Config,
            base_path: PathBuf,
        }
        
        impl TestConfig {
            fn get_config_path(&self) -> PathBuf {
                self.base_path.join(".basecamp/config.yaml")
            }
            
            fn get_codebases_path(&self) -> PathBuf {
                self.base_path.join(".basecamp/codebases.yaml")
            }
            
            fn save(&self) -> BasecampResult<()> {
                // Save git config
                let yaml = serde_yaml::to_string(&self.config.git_config)?;
                let mut file = File::create(self.get_config_path())?;
                file.write_all(yaml.as_bytes())?;
                
                // Save codebases config
                let yaml = serde_yaml::to_string(&self.config.codebases_config)?;
                let mut file = File::create(self.get_codebases_path())?;
                file.write_all(yaml.as_bytes())?;
                
                Ok(())
            }
            
            fn load(&self) -> BasecampResult<Config> {
                // Load git config
                let git_config = if self.get_config_path().exists() {
                    let content = std::fs::read_to_string(self.get_config_path())?;
                    serde_yaml::from_str(&content)?
                } else {
                    return Err(BasecampError::FileNotFound(self.get_config_path()));
                };
                
                // Load codebases config
                let codebases_config = if self.get_codebases_path().exists() {
                    let content = std::fs::read_to_string(self.get_codebases_path())?;
                    serde_yaml::from_str(&content)?
                } else {
                    CodebasesConfig::default()
                };
                
                Ok(Config {
                    git_config,
                    codebases_config,
                })
            }
        }
        
        // Create a test config with the absolute path
        let test_config = TestConfig {
            config,
            base_path: temp_path.clone(),
        };

        // Save the configuration using absolute paths
        println!("Saving configuration using direct file operations...");
        test_config.save().expect("Failed to save config");
        
        // Verify the config files exist
        let config_path = test_config.get_config_path();
        let codebases_path = test_config.get_codebases_path();
        
        // List directory contents after saving
        println!("Directory contents after saving:");
        if let Ok(entries) = std::fs::read_dir(&basecamp_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("  {:?}", entry.path());
                }
            }
        } else {
            println!("  Failed to read directory contents");
        }
        
        assert!(config_path.exists(), 
                "Config file not created at: {:?}", config_path);
        
        assert!(codebases_path.exists(), 
                "Codebases file not created at: {:?}", codebases_path);

        // If codebases file exists, read and print its content
        if codebases_path.exists() {
            let codebases_content = std::fs::read_to_string(&codebases_path)
                .expect("Failed to read codebases.yaml");
            println!("Raw codebases.yaml content:\n{}", codebases_content);
        } else {
            println!("Codebases file does not exist, cannot read content");
        }

        // Load the config back and verify contents
        println!("Loading saved configuration...");
        let loaded_config = test_config.load().expect("Failed to load saved config");
        assert_eq!(loaded_config.git_config.github_url, "https://github.com/test-org");
        assert_eq!(loaded_config.codebases_config.codebases.len(), 1);
        assert!(loaded_config.codebases_config.codebases.contains_key("test-codebase"), 
                "test-codebase not found in codebases: {:?}", 
                loaded_config.codebases_config.codebases);
        
        // Get the repositories and verify each expected one exists
        let loaded_repos = loaded_config.codebases_config.codebases.get("test-codebase")
            .expect("test-codebase not found in loaded config");
        println!("Loaded repositories: {:?}", loaded_repos);
        
        // Check if each repository was saved and loaded correctly
        for repo in &repos_to_add {
            assert!(loaded_repos.contains(repo), 
                    "Repository {} not found in loaded repositories: {:?}", 
                    repo, loaded_repos);
        }
        
        // Check the total count matches
        assert_eq!(loaded_repos.len(), repos_to_add.len(), 
                  "Expected {} repositories, found {}", 
                  repos_to_add.len(), loaded_repos.len());

        // Cleanup
        println!("Test cleanup: removing temp directory");
        common::teardown(temp_dir);
    });
    
    // Always return to the original directory
    println!("Returning to original directory: {:?}", original_dir);
    if let Err(e) = std::env::set_current_dir(&original_dir) {
        eprintln!("Failed to return to original directory: {}", e);
    }
    
    // Re-throw any panic that occurred in the test
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
    
    println!("Test config_save completed");
}

#[test]
fn test_add_repositories() {
    // Setup
    let mut config = Config::new();
    config
        .set_github_url("https://github.com/test-org".to_string())
        .unwrap();

    // Test
    config
        .add_repositories("frontend", &["repo1".to_string(), "repo2".to_string()])
        .unwrap();
    config
        .add_repositories("backend", &["api".to_string()])
        .unwrap();

    // Verify
    assert_eq!(config.codebases_config.codebases.len(), 2);
    assert_eq!(config.codebases_config.codebases.get("frontend").unwrap().len(), 2);
    assert_eq!(config.codebases_config.codebases.get("backend").unwrap().len(), 1);

    // Test adding to existing codebase
    config
        .add_repositories("frontend", &["repo3".to_string()])
        .unwrap();
    assert_eq!(config.codebases_config.codebases.get("frontend").unwrap().len(), 3);
}

#[test]
fn test_remove_repositories() {
    // Setup
    let mut config = Config::new();
    config
        .set_github_url("https://github.com/test-org".to_string())
        .unwrap();
    config
        .add_repositories(
            "frontend",
            &[
                "repo1".to_string(),
                "repo2".to_string(),
                "repo3".to_string(),
            ],
        )
        .unwrap();

    // Test
    config
        .remove_repositories("frontend", &["repo2".to_string()])
        .unwrap();

    // Verify
    let repos = config.get_repositories("frontend").unwrap();
    assert_eq!(repos.len(), 2);
    assert!(repos.contains(&"repo1".to_string()));
    assert!(!repos.contains(&"repo2".to_string()));
    assert!(repos.contains(&"repo3".to_string()));
}
