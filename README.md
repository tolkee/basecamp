  <img src="logo.png" alt="BaseCamp Logo" width="70"/>

# BaseCamp

A streamlined CLI tool for managing multiple codebases and repositories.

BaseCamp helps you organize and work with multiple related Git repositories, making it easy to:

- Define and manage codebases (logical groupings of repositories)
- Initialize development environments quickly
- Clone multiple repositories in parallel
- Add or remove repositories from codebases
- Track and list repositories across projects

## Installation

### From Source

```bash
# Clone the repository
git clone git@github.com:tolkee/basecamp.git
cd basecamp

# Build and install
cargo install --path .
```

### Future Distribution Methods

We're working on providing BaseCamp through:

- Homebrew: `brew install basecamp`
- APT: `apt install basecamp`
- Cargo: `cargo install basecamp`

## Usage

### Initialize BaseCamp

```bash
basecamp init
```

### Add Repositories to a Codebase

```bash
# Add one or more repositories to a codebase
basecamp add frontend react-app dashboard settings
```

### Install Repositories

```bash
# Clone all repositories in all codebases
basecamp install

# Clone all repositories in a specific codebase
basecamp install frontend

# Control parallel clone operations
basecamp install --parallel 8
```

### List Codebases and Repositories

```bash
# List all codebases
basecamp list

# List repositories in a specific codebase
basecamp list frontend
```

### Remove Repositories or Codebases

```bash
# Remove specific repositories from a codebase
basecamp remove frontend settings

# Remove an entire codebase
basecamp remove frontend

# Force removal even if there are uncommitted changes
basecamp remove frontend --force
```

## Configuration

BaseCamp uses a `.basecamp` directory in your project root to store configuration:

- `config.yaml`: Git configuration including GitHub URL
- `codebases.yaml`: Configuration of codebases and repositories

The settings are stored as:

```yaml
# config.yaml
github_url: https://github.com/tolkee
```

```yaml
# codebases.yaml
codebases:
  frontend:
    - react-app
    - dashboard
    - settings
  backend:
    - api-server
    - auth-service
    - database
```

## Development

### Requirements

- Rust 1.70 or higher
- Git

### Building from Source

```bash
cargo build --release
```

### Running Tests

```bash
cargo test
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
