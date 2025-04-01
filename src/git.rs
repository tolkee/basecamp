use git2::{Repository, StatusOptions, RemoteCallbacks, FetchOptions, build::RepoBuilder, Cred, ErrorCode};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use std::env;

use crate::error::{BasecampError, BasecampResult};

/// Git repository operations
pub struct GitRepo;

impl GitRepo {
    /// Clone a Git repository to the specified path
    pub fn clone(url: &str, path: &Path) -> BasecampResult<Repository> {
        debug!("Cloning repository {} to {:?}", url, path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Determine if this is an SSH URL
        let is_ssh_url = url.starts_with("git@");
        let username = if is_ssh_url {
            // Extract username from git@github.com:user/repo
            url.split('@').nth(1)
                .and_then(|s| s.split(':').next())
                .unwrap_or("git")
        } else {
            "git"
        };

        // Set up authentication callbacks for SSH
        let mut callbacks = RemoteCallbacks::new();
        
        // Track authentication attempts to prevent infinite loops
        let attempt_count = std::cell::Cell::new(0);
        
        callbacks.credentials(move |_url, username_from_url, allowed_types| {
            let current_attempt = attempt_count.get();
            attempt_count.set(current_attempt + 1);
            
            // Prevent too many authentication attempts
            if current_attempt > 5 {
                warn!("Too many authentication attempts, giving up");
                return Err(git2::Error::from_str("Too many authentication attempts"));
            }
            
            let username = username_from_url.unwrap_or(username);
            debug!("Authentication attempt #{} for user: {}", current_attempt + 1, username);
            
            // Check if HTTPS authentication is requested
            if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                debug!("HTTP authentication requested, using default credentials");
                return Cred::default();
            }
            
            // Only try SSH agent on first attempt to avoid prompting multiple times
            if current_attempt == 0 {
                debug!("Trying SSH agent");
                if let Ok(cred) = Cred::ssh_key_from_agent(username) {
                    debug!("Found credentials in SSH agent");
                    return Ok(cred);
                }
            }
            
            // Find SSH keys in the standard locations
            let home = env::var("HOME").unwrap_or_else(|_| "~".to_string());
            let ssh_path = Path::new(&home).join(".ssh");
            
            // Try to get a list of all key files in .ssh directory
            let mut key_attempts = Vec::new();
            
            // Standard key types to try (with paths)
            key_attempts.push((ssh_path.join("id_ed25519"), ssh_path.join("id_ed25519.pub")));
            key_attempts.push((ssh_path.join("id_rsa"), ssh_path.join("id_rsa.pub")));
            key_attempts.push((ssh_path.join("id_ecdsa"), ssh_path.join("id_ecdsa.pub")));
            key_attempts.push((ssh_path.join("id_dsa"), ssh_path.join("id_dsa.pub")));
            
            // Add GitHub specific keys
            key_attempts.push((ssh_path.join("github_rsa"), ssh_path.join("github_rsa.pub")));
            key_attempts.push((ssh_path.join("github_ed25519"), ssh_path.join("github_ed25519.pub")));
            
            // Try to find keys from SSH config
            if let Ok(config_content) = std::fs::read_to_string(ssh_path.join("config")) {
                for line in config_content.lines() {
                    if line.trim().starts_with("IdentityFile") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let identity_path_str = parts[1].replace("~", &home);
                            let identity_path = PathBuf::from(&identity_path_str);
                            let pub_identity_path = PathBuf::from(format!("{}.pub", identity_path_str));
                            
                            key_attempts.push((identity_path, pub_identity_path));
                        }
                    }
                }
            }

            // Try to list all files in .ssh directory and find potential keys
            if let Ok(entries) = std::fs::read_dir(&ssh_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        if !filename.contains(".pub") && !filename.starts_with(".") && !filename.contains("known_hosts") && !filename.contains("config") {
                            let pub_path = path.with_extension("pub");
                            if pub_path.exists() {
                                key_attempts.push((path.clone(), pub_path));
                            } else {
                                // Some keys might not have .pub extension explicitly
                                let pub_path2 = PathBuf::from(format!("{}.pub", path.to_string_lossy()));
                                if pub_path2.exists() {
                                    key_attempts.push((path.clone(), pub_path2));
                                }
                            }
                        }
                    }
                }
            }
            
            // We want to try a different key on each authentication attempt
            // after the first SSH agent attempt
            let adjusted_attempt = if current_attempt == 0 { 0 } else { current_attempt - 1 };
            let key_index = adjusted_attempt as usize % key_attempts.len();
            
            // Try the selected key
            if key_index < key_attempts.len() {
                let (key_path, pub_key_path) = &key_attempts[key_index];
                
                if key_path.exists() {
                    debug!("Trying key {}/{}: {:?}", key_index + 1, key_attempts.len(), key_path);
                    
                    // Try with public key
                    if pub_key_path.exists() {
                        if let Ok(cred) = Cred::ssh_key(username, Some(pub_key_path), key_path, None) {
                            return Ok(cred);
                        }
                    }
                    
                    // Try without public key
                    if let Ok(cred) = Cred::ssh_key(username, None, key_path, None) {
                        return Ok(cred);
                    }
                    
                    // If we're still here, the key might require a passphrase
                    // Unfortunately, git2 doesn't provide a way to prompt for passphrase interactively
                    warn!("Key {:?} might require a passphrase. Consider adding it to your SSH agent first with: ssh-add {:?}", key_path, key_path);
                }
            }
            
            // If we've tried all keys and still here, fallback to default which will likely fail
            warn!("Couldn't authenticate with any available SSH key. Ensure your SSH keys are set up correctly.");
            Cred::default()
        });

        // Set up fetch options with callbacks
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        // Use RepoBuilder with fetch options
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        // Clone the repository with auth settings
        let repo = match builder.clone(url, path) {
            Ok(repo) => repo,
            Err(e) => {
                warn!("Failed to clone repository: {}", e);
                
                // Provide more helpful error messages for SSH issues
                if is_ssh_url && (e.code() == ErrorCode::Auth || e.class() == git2::ErrorClass::Ssh) {
                    warn!("SSH authentication failed. Here are some troubleshooting steps:");
                    warn!("1. Check if your SSH key is set up correctly: ssh -T git@github.com");
                    warn!("2. Try adding your key to the SSH agent: ssh-add ~/.ssh/id_ed25519");
                    warn!("3. Verify your GitHub URL format is correct: git@github.com:username/repo.git");
                    
                    if e.message().contains("passphrase") {
                        warn!("4. Your SSH key appears to be protected with a passphrase.");
                        warn!("   Please add it to your SSH agent first: ssh-add ~/.ssh/id_ed25519");
                    }
                }
                
                return Err(BasecampError::GitError(e));
            }
        };

        info!("Repository cloned successfully to {:?}", path);
        Ok(repo)
    }

    /// Check if a repository has uncommitted changes
    pub fn has_uncommitted_changes(repo_path: &Path) -> BasecampResult<bool> {
        debug!("Checking for uncommitted changes in {:?}", repo_path);

        let repo = Repository::open(repo_path)?;
        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = repo.statuses(Some(&mut status_opts))?;

        if !statuses.is_empty() {
            debug!("Found {} uncommitted changes", statuses.len());
            return Ok(true);
        }

        Ok(false)
    }

    /// Check if a repository has unpushed commits
    pub fn has_unpushed_commits(repo_path: &Path) -> BasecampResult<bool> {
        debug!("Checking for unpushed commits in {:?}", repo_path);

        let repo = Repository::open(repo_path)?;

        // Get the current branch
        let head = repo.head()?;
        let branch_name = head.shorthand().unwrap_or("HEAD");

        // Find remote tracking branch
        let remote_branch =
            match repo.find_branch(&format!("origin/{}", branch_name), git2::BranchType::Remote) {
                Ok(branch) => branch,
                Err(_) => {
                    debug!("No remote tracking branch found for {}", branch_name);
                    return Ok(false); // No remote branch to compare with
                }
            };

        // Get commits for comparison
        let local_commit = head.peel_to_commit()?;
        let remote_commit = remote_branch.get().peel_to_commit()?;

        // Check if local is ahead of remote
        let local_id = local_commit.id();
        let remote_id = remote_commit.id();

        if local_id != remote_id {
            // Additional check could be done here with git2::graph_ahead_behind
            // to count exactly how many commits ahead/behind
            debug!("Local branch is different from remote branch");
            return Ok(true);
        }

        Ok(false)
    }

    /// Build a repository URL from the GitHub base URL and repository name
    pub fn build_repo_url(github_url: &str, repo_name: &str) -> String {
        // Handle both https and git@ URL formats
        if github_url.starts_with("https://") {
            // Ensure URL ends with a slash
            let base_url = if github_url.ends_with('/') {
                github_url.to_string()
            } else {
                format!("{}/", github_url)
            };

            format!("{}{}.git", base_url, repo_name)
        } else if github_url.starts_with("git@") {
            // Handle SSH format
            let parts: Vec<&str> = github_url.split(':').collect();
            if parts.len() == 2 {
                let host = parts[0];
                let path = if parts[1].ends_with('/') {
                    parts[1]
                } else {
                    &format!("{}/", parts[1])
                };
                format!("{}:{}{}.git", host, path, repo_name)
            } else {
                // Fallback for malformed URLs
                format!("{}/{}.git", github_url, repo_name)
            }
        } else {
            // Fallback for other formats
            format!("{}/{}.git", github_url, repo_name)
        }
    }

    /// Get the path for a repository in a specific codebase
    pub fn get_repo_path(codebase: &str, repo_name: &str) -> PathBuf {
        PathBuf::from(codebase).join(repo_name)
    }
}
