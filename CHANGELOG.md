# Changelog

All notable changes to the BaseCamp project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2024-04-01

### Added

- Enhanced UI with arrow-key navigation for the `init` command
- New `select` method in the UI class utilizing dialoguer's Select component

### Changed

- Removed legacy configuration handling with `basecamp.yaml`
- Transitioned to a new configuration structure using `.basecamp` directory
- Simplified the `init` command by removing the `github_url` parameter
- Enhanced `remove` command to delete local folders on disk automatically
- Improved confirmation messages for file deletion operations

### Fixed

- Fixed test failures related to configuration loading
- Removed unused code to eliminate dead code warnings
- Added `#[allow(dead_code)]` attribute to unused UI methods

## [0.2.0] - Unreleased

### Added

- Configuration management via `.basecamp` directory structure
- Support for multiple codebases and repositories

### Changed

- Improved CLI interface with better error messages
- Enhanced Git operations with better error handling

## [0.1.2] - Initial Release

### Added

- Basic CLI functionality for managing codebases
- Git repository operations
- Simple configuration management with YAML files
