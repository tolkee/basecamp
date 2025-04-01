use clap::{Parser, Subcommand};

/// BaseCamp: A streamlined tool for managing multiple codebases and repositories
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
pub struct Cli {
    /// Verbosity level (-v, -vv, -vvv)
    #[clap(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Subcommands
    #[clap(subcommand)]
    pub command: Commands,
}

/// BaseCamp subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new BaseCamp configuration
    Init {
        /// Connection type: 'https' or 'ssh'
        #[clap(long)]
        connection_type: Option<String>,
        
        /// Repository type: 'org' or 'personal'
        #[clap(long)]
        repo_type: Option<String>,
        
        /// Organization name or GitHub username
        #[clap(long)]
        name: Option<String>,
        
        /// Non-interactive mode
        #[clap(long)]
        non_interactive: bool,
        
        /// Force overwrite existing configuration
        #[clap(long)]
        force: bool,
    },

    /// Install all repositories for all codebases or a specific codebase
    Install {
        /// Codebase name (if not specified, all codebases will be installed)
        codebase: Option<String>,

        /// Number of parallel clone operations
        #[clap(short, long, default_value = "4")]
        parallel: usize,
    },

    /// List all codebases or repositories in a specific codebase
    List {
        /// Codebase name (if not specified, all codebases will be listed)
        codebase: Option<String>,
    },

    /// Add repositories to a codebase
    Add {
        /// Codebase name
        codebase: String,

        /// Repository names
        #[clap(required = true)]
        repositories: Vec<String>,
    },

    /// Remove repositories from a codebase or remove an entire codebase
    Remove {
        /// Codebase name
        codebase: String,

        /// Repository names (if not specified, the entire codebase will be removed)
        repositories: Vec<String>,

        /// Force removal even if there are uncommitted changes
        #[clap(short, long)]
        force: bool,
    },
}

/// Parse command-line arguments
pub fn parse_args() -> Cli {
    Cli::parse()
}
