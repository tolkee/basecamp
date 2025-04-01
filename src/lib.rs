/*!
# BaseCamp

BaseCamp is a command-line utility for managing multiple related Git repositories organized into logical
codebases. It allows developers to quickly set up and manage development environments across projects.

## Features

- **Codebase Management**: Group related repositories into logical codebases
- **Easy Setup**: Initialize development environments by cloning all repositories
- **Parallel Operations**: Clone multiple repositories simultaneously
- **Safety Checks**: Detect uncommitted changes and unpushed commits before removal
- **Rich UI**: Colorful output with progress indicators

## Command Overview

- `init`: Create a new BaseCamp configuration
- `install`: Clone repositories for a codebase
- `list`: Display codebases and repositories
- `add`: Add repositories to a codebase
- `remove`: Remove repositories or entire codebases

## Usage Example

```bash
# Initialize with GitHub URL
basecamp init --github-url https://github.com/your-org

# Add repositories to a codebase
basecamp add frontend react-app dashboard settings

# Clone all repositories
basecamp install

# List all codebases and their repositories
basecamp list
```

## Modules

The crate is organized into several modules:

- [`cli`]: Command-line interface and argument parsing
- [`commands`]: Implementation of the main commands
- [`config`]: Configuration loading, saving, and manipulation
- [`error`]: Error handling types
- [`git`]: Git operations including cloning and status checks
- [`logger`]: Logging setup
- [`ui`]: Terminal UI utilities including progress bars and colored output
*/

pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod git;
pub mod logger;
pub mod ui;
