# BaseCamp: Migration Plan from Shell Prototype to Rust CLI

## Overview

This document outlines the plan to transform the BaseCamp shell prototype into a robust, cross-platform Rust CLI application that can be distributed through package managers like Homebrew and APT.

## 1. Understanding the Current Prototype

The shell prototype offers the following functionality:

- **Configuration Management**: Uses a YAML configuration file (`basecamp.yaml`) to define codebases and repositories
- **Repository Management**:
  - Initializing a development environment by cloning all repositories
  - Adding repositories to codebases
  - Removing repositories or entire codebases
  - Listing repositories in specific or all codebases
- **Safety Checks**: Detects uncommitted changes and unpushed commits before removal operations

## 2. Architecture Design for Rust CLI

### Core Components

1. **CLI Interface**

   - Use [clap](https://crates.io/crates/clap) for command-line argument parsing
   - Implement subcommands that match the shell prototype with enhancements (`init`, `install`, `list`, `add`, `remove`, `help`)
   - Create rich, colorful help messages with detailed command descriptions
   - Add command examples and usage patterns

2. **Configuration Management**

   - Use [serde](https://crates.io/crates/serde) and [serde_yaml](https://crates.io/crates/serde_yaml) for YAML parsing
   - Create Rust structs to represent the configuration schema
   - Include validation for configuration files

3. **Git Operations**

   - Use [git2](https://crates.io/crates/git2) for Git operations (cloning, status checks)
   - Implement proper error handling for Git operations
   - Add progress reporting for long-running Git operations

4. **File System Operations**

   - Use Rust's standard library for file system operations
   - Implement cross-platform path handling
   - Include permission checking and error recovery

5. **Error Handling**

   - Implement robust error handling with custom error types
   - Provide meaningful error messages to users
   - Include context-aware suggestions for resolving common errors

6. **Logging & Output**
   - Use [log](https://crates.io/crates/log) and [env_logger](https://crates.io/crates/env_logger) for logging
   - Add [console](https://crates.io/crates/console) for terminal coloring and formatting
   - Implement different verbosity levels (error, warn, info, debug, trace)
   - Use [indicatif](https://crates.io/crates/indicatif) for progress bars and spinners

## 3. Data Structures

```rust
// Configuration data structures
struct Config {
    github_url: String,
    codebases: HashMap<String, Vec<String>>,
}

// Command enums
enum Command {
    Init,
    Install,
    List { codebase: Option<String> },
    Add { codebase: String, repos: Vec<String> },
    Remove { codebase: String, repos: Vec<String> },
    Help,
}

// Result/Error types
enum BasecampError {
    ConfigError(String),
    GitError(git2::Error),
    FileSystemError(std::io::Error),
    // ...
}
```

## 4. Implementation Plan

### Phase 1: Setup Project Structure

1. **Initialize Rust Project**

   ```bash
   cargo new basecamp --bin
   ```

2. **Define Dependencies in Cargo.toml**

   - clap (with derive and color features)
   - serde, serde_yaml
   - git2
   - log, env_logger
   - anyhow/thiserror for error handling
   - console for terminal styling
   - indicatif for progress indicators
   - similar (for typo suggestions)
   - prettytable-rs for formatted table output
   - dialoguer for interactive prompts

3. **Create Directory Structure**
   ```
   src/
   ├── main.rs           # Entry point
   ├── cli.rs            # CLI argument parsing
   ├── config.rs         # Configuration loading/saving
   ├── commands/         # Command implementations
   │   ├── mod.rs
   │   ├── init.rs       # Initial setup and config creation
   │   ├── install.rs    # Repository cloning and environment setup
   │   ├── list.rs
   │   ├── add.rs
   │   └── remove.rs
   ├── git.rs            # Git operations
   ├── logger.rs         # Logging setup and configuration
   ├── ui.rs             # User interface utilities
   └── error.rs          # Error handling
   tests/                # Integration tests
   ├── common/           # Shared test utilities
   ├── init_tests.rs
   ├── install_tests.rs
   ├── list_tests.rs
   ├── add_tests.rs
   └── remove_tests.rs
   ```

### Phase 2: Core Functionality Implementation

1. **Config Management**

   - Parse and validate basecamp.yaml
   - Implement saving configuration changes
   - Add schema validation with helpful error messages

2. **Command Implementation**

   - **Init Command**
     - Interactive prompt for GitHub URL configuration
     - Create initial basecamp.yaml file
     - Validate GitHub URL format
   - Repository cloning with progress indicators
   - Status checking for uncommitted changes
   - Unpushed commit detection
   - Parallel clone operations with proper progress reporting

3. **Command Implementation**

   - Implement each command (init, list, add, remove)
   - Ensure parity with shell prototype
   - Add colorful, formatted output for each command
   - Implement table-based output for list command

4. **Error Handling**

   - Implement custom error types
   - Add friendly error messages
   - Include suggestions for fixing common errors
   - Implement error context and chaining

5. **User Interface**
   - Implement consistent color scheme
   - Add spinners for long-running operations
   - Create progress bars for multi-step processes
   - Design table formats for data display

### Phase 3: Comprehensive Testing

1. **Unit Tests**

   - Test each component in isolation
   - Implement mocks for external dependencies (Git, filesystem)
   - Test error conditions and edge cases
   - Aim for >90% code coverage

2. **Integration Tests**

   - Test end-to-end workflows
   - Use temporary directories for filesystem tests
   - Mock Git repositories for testing clone/status operations
   - Test CLI argument parsing and execution
   - Validate output formatting and colors

3. **Property-Based Testing**

   - Use [proptest](https://crates.io/crates/proptest) for generating test cases
   - Test config parsing with randomly generated YAML
   - Verify correct handling of malformed inputs

4. **Cross-Platform Testing**

   - Test on Linux, macOS, and Windows
   - Verify path handling works correctly across platforms
   - Ensure terminal colors work on different terminal emulators
   - Test with various Git versions

5. **Performance Testing**
   - Benchmark operations with large numbers of repositories
   - Verify parallel operations scale appropriately
   - Test memory usage with large configurations

### Phase 4: Documentation and Packaging

1. **Code Documentation**

   - Add comprehensive rustdoc comments
   - Include examples in documentation
   - Document error conditions and recovery

2. **User Documentation**

   - Create detailed README with installation instructions
   - Add usage examples with screenshots
   - Create a user guide with common workflows
   - Include troubleshooting section

3. **Binary Packaging**
   - Optimize binary size
   - Set up release process
   - Create installers for different platforms

## 5. Distribution Plan

### Homebrew (macOS)

1. **Create Homebrew Formula**

   ```ruby
   class Basecamp < Formula
     desc "A streamlined tool for managing multiple codebases and repositories"
     homepage "https://github.com/your-org/basecamp"
     url "https://github.com/your-org/basecamp/releases/download/v1.0.0/basecamp-macos.tar.gz"
     sha256 "..."

     def install
       bin.install "basecamp"
     end

     test do
       system "#{bin}/basecamp", "--version"
     end
   end
   ```

2. **Submit to Homebrew Core** or maintain a tap

### APT (Debian/Ubuntu)

1. **Create Debian Package**

   - Use [cargo-deb](https://crates.io/crates/cargo-deb) to build .deb package
   - Include man pages and bash completion

2. **Set Up PPA Repository**
   - Create and maintain a PPA for easy installation

### Other Distribution Methods

1. **Cargo Install**

   ```bash
   cargo install basecamp
   ```

2. **Direct Binary Downloads**

   - Provide pre-built binaries for different platforms
   - Use GitHub Releases for distribution
   - Include checksums for verification

3. **Windows Package Manager (Chocolatey)**
   - Create a Chocolatey package

## 6. CI/CD Pipeline

1. **GitHub Actions Workflow**

   - Automated testing on multiple platforms
   - Run tests with different Rust versions
   - Generate test coverage reports
   - Automatic binary building for releases
   - Package generation for different platforms

2. **Release Process**
   - Semantic versioning
   - Changelog generation
   - Automated package deployment
   - Release notes with feature highlights

## 7. CLI User Experience Improvements

1. **Rich Help System**

   - Colorful, well-formatted help messages
   - Command examples for common use cases
   - Context-sensitive help for subcommands
   - ASCII art logo in help message

2. **Interactive Features**

   - Use [dialoguer](https://crates.io/crates/dialoguer) for interactive prompts
   - Add confirmation dialogues for destructive operations
   - Implement interactive repository selection
   - Add autocompletion for repository names

3. **Beautiful Output Formatting**

   - Color-coded output based on message type
   - Use emojis for status indicators
   - Format tabular data with [prettytable-rs](https://crates.io/crates/prettytable-rs)
   - Output JSON/YAML for machine-readable results when requested

4. **Progress Visualization**

   - Show progress bars for clone operations
   - Display spinners during status checks
   - Multi-progress displays for parallel operations
   - ETA and throughput estimates for long-running tasks

5. **Logging Enhancements**
   - Different log levels with appropriate coloring
   - Structured logging for machine consumption
   - Log file output option
   - Configurable verbosity

## 8. Performance Improvements

1. **Parallel Operations**

   - Clone multiple repositories simultaneously
   - Parallel status checks across repositories
   - Throttling to prevent overwhelming the system

2. **Caching**

   - Cache repository status to speed up repeated commands
   - Cache remote repository information

3. **Optimizations**
   - Lazy loading of repository information
   - Incremental updates to configuration

## 9. Next Steps

1. Set up initial Rust project structure
2. Implement configuration parsing
3. Create testing framework and initial tests
4. Port each shell command to Rust, starting with the simplest ones
5. Design and implement the user interface components
6. Set up CI/CD pipeline for automated testing and building
