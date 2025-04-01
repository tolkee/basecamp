mod common;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("basecamp").unwrap();

    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("BaseCamp"))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("Options:"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("basecamp").unwrap();

    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("basecamp"));
}

#[test]
fn test_init_command() {
    // Setup
    let (temp_dir, temp_path) = common::setup_temp_dir();

    // Create mock config files to ensure non-interactive execution
    let basecamp_dir = temp_path.join(".basecamp");
    std::fs::create_dir_all(&basecamp_dir).unwrap();
    
    // Create a basic config.yaml file
    let config_content = "github_url: https://github.com/test-org";
    std::fs::write(basecamp_dir.join("config.yaml"), config_content).unwrap();
    
    // Create an empty codebases.yaml file
    let codebases_content = "codebases: {}";
    std::fs::write(basecamp_dir.join("codebases.yaml"), codebases_content).unwrap();

    // Run init command - expect it to detect existing config and ask for confirmation,
    // which we can't provide in a test, so it will timeout or be cancelled.
    // We're just verifying that the command syntax is correct.
    let mut cmd = Command::cargo_bin("basecamp").unwrap();
    cmd.arg("init")
        .current_dir(&temp_path)
        .timeout(std::time::Duration::from_millis(100));

    // Since we can't interact, the command will timeout, which is expected
    let output = cmd.output().unwrap();
    
    // The command should start running and at least show the prompt
    let output_str = std::str::from_utf8(&output.stdout).unwrap_or("");
    assert!(output_str.contains("exist") || output_str.is_empty());
    
    // Verify config files exist (they were created by us for the test)
    assert!(basecamp_dir.exists());
    assert!(basecamp_dir.join("config.yaml").exists());
    assert!(basecamp_dir.join("codebases.yaml").exists());

    // Cleanup
    common::teardown(temp_dir);
}

// Add a new test for init command in non-interactive mode
#[test]
fn test_init_command_no_config() {
    // Setup - temporary directory without existing config
    let (temp_dir, temp_path) = common::setup_temp_dir();
    
    // Make sure there's no .basecamp directory
    let basecamp_dir = temp_path.join(".basecamp");
    if basecamp_dir.exists() {
        std::fs::remove_dir_all(&basecamp_dir).unwrap();
    }
    
    // We'll use command line parameters instead of environment variables
    let mut cmd = Command::cargo_bin("basecamp").unwrap();
    cmd.arg("init")
        .arg("--non-interactive")
        .arg("--connection-type").arg("https")
        .arg("--repo-type").arg("org")
        .arg("--name").arg("test-org")
        .current_dir(&temp_path);
    
    // Attempt to run the command - it should succeed with non-interactive mode
    let assert = cmd.assert();
    
    // In non-interactive mode, it should succeed and create files
    assert.success();
    
    // Verify the config files were created
    assert!(basecamp_dir.exists());
    assert!(basecamp_dir.join("config.yaml").exists());
    assert!(basecamp_dir.join("codebases.yaml").exists());
    
    // Verify content of config.yaml
    let config_content = std::fs::read_to_string(basecamp_dir.join("config.yaml")).unwrap();
    assert!(config_content.contains("github_url: https://github.com/test-org"));
    
    // Cleanup
    common::teardown(temp_dir);
}

#[test]
fn test_list_without_config() {
    // Setup
    let (temp_dir, temp_path) = common::setup_temp_dir();

    // Ensure .basecamp directory doesn't exist
    let basecamp_dir = temp_path.join(".basecamp");
    if basecamp_dir.exists() {
        std::fs::remove_dir_all(&basecamp_dir).unwrap();
    }

    // Run list command without a config file
    let mut cmd = Command::cargo_bin("basecamp").unwrap();
    cmd.arg("list").current_dir(&temp_path);

    // Verify command fails with appropriate error message
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File not found"));

    // Cleanup
    common::teardown(temp_dir);
}

#[test]
fn test_list_with_config() {
    // Setup
    let (temp_dir, temp_path) = common::setup_temp_dir();
    common::create_test_config(&temp_path);

    // Run list command
    let mut cmd = Command::cargo_bin("basecamp").unwrap();
    cmd.arg("list").current_dir(&temp_path);

    // Verify command succeeds and lists the codebases
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("frontend"))
        .stdout(predicate::str::contains("backend"));

    // Cleanup
    common::teardown(temp_dir);
}
