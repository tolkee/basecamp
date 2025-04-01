# Changelog

All notable changes to the BaseCamp project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-04-01

Initial release of BaseCamp CLI tool with the following features:

- Interactive command-line interface with arrow-key navigation
- Configuration management via `.basecamp` directory structure
- Multiple codebase and repository management
- Git operation support for cloning and managing repositories
- `init` command for setting up a new BaseCamp environment
- `add` command for adding repositories to a codebase
- `list` command for viewing codebases and their repositories
- `install` command for cloning repositories
- `remove` command for removing repositories and codebases (with local files)
- Colorful terminal UI with progress indicators
- Safety checks for uncommitted changes and unpushed commits
