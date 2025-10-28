use std::path::PathBuf;
use std::process::Command;
use crate::error::{LauncherError, Result};
use crate::utils::ProgressTracker;

/// GitHub repository information
pub struct GitHubRepo {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

impl GitHubRepo {
    /// Create a new GitHub repo reference
    pub fn new(owner: impl Into<String>, repo: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            owner: owner.into(),
            repo: repo.into(),
            branch: branch.into(),
        }
    }

    /// Get the clone URL
    pub fn clone_url(&self) -> String {
        format!("https://github.com/{}/{}.git", self.owner, self.repo)
    }
}

/// Git-based installer (matches old INSTALL_OR_UPDATE.bat behavior)
pub struct GitInstaller {
    repo: GitHubRepo,
}

impl GitInstaller {
    /// Create a new git installer
    pub fn new(repo: GitHubRepo) -> Self {
        Self { repo }
    }

    /// Install or update game using git
    pub async fn install_or_update(
        &self,
        install_path: &PathBuf,
        progress: Option<ProgressTracker>,
    ) -> Result<()> {
        // Ensure install directory exists
        std::fs::create_dir_all(install_path)?;

        // Check if .git directory exists
        let git_dir = install_path.join(".git");
        let is_existing = git_dir.exists();

        if is_existing {
            // Update existing installation
            self.update_existing(install_path, progress).await
        } else {
            // Fresh installation
            self.fresh_install(install_path, progress).await
        }
    }

    /// Perform fresh installation
    async fn fresh_install(
        &self,
        install_path: &PathBuf,
        progress: Option<ProgressTracker>,
    ) -> Result<()> {
        if let Some(ref p) = progress {
            p.update(10);
        }

        // Initialize git repository
        tracing::info!("Initializing git repository...");
        self.run_git_command(install_path, &["init", "."])?;

        if let Some(ref p) = progress {
            p.update(20);
        }

        // Add remote
        tracing::info!("Adding remote origin...");
        let remote_url = self.repo.clone_url();
        self.run_git_command(install_path, &["remote", "add", "origin", &remote_url])?;

        if let Some(ref p) = progress {
            p.update(30);
        }

        // Fetch with depth=1 for efficiency (this is the slow part)
        tracing::info!("Fetching from GitHub (this may take a few minutes)...");
        self.run_git_command(
            install_path,
            &["fetch", "--depth=1", "origin", &self.repo.branch, "--progress"],
        )?;

        if let Some(ref p) = progress {
            p.update(80);
        }

        // Reset to fetched branch
        tracing::info!("Applying files...");
        let reset_ref = format!("origin/{}", self.repo.branch);
        self.run_git_command(install_path, &["reset", "--hard", &reset_ref])?;

        if let Some(ref p) = progress {
            p.update(100);
        }

        tracing::info!("Installation complete!");
        Ok(())
    }

    /// Update existing installation
    async fn update_existing(
        &self,
        install_path: &PathBuf,
        progress: Option<ProgressTracker>,
    ) -> Result<()> {
        // Clean up shallow lock if it exists
        let shallow_lock = install_path.join(".git").join("shallow.lock");
        if shallow_lock.exists() {
            let _ = std::fs::remove_file(&shallow_lock);
        }

        if let Some(ref p) = progress {
            p.update(20);
        }

        // Fetch latest changes (this is the slow part)
        tracing::info!("Fetching updates from GitHub (this may take a few minutes)...");
        self.run_git_command(
            install_path,
            &["fetch", "--depth=1", "origin", &self.repo.branch, "--progress"],
        )?;

        if let Some(ref p) = progress {
            p.update(70);
        }

        // Reset to latest
        tracing::info!("Applying updates...");
        let reset_ref = format!("origin/{}", self.repo.branch);
        self.run_git_command(install_path, &["reset", "--hard", &reset_ref])?;

        if let Some(ref p) = progress {
            p.update(100);
        }

        tracing::info!("Update complete!");
        Ok(())
    }

    /// Run a git command
    fn run_git_command(&self, cwd: &PathBuf, args: &[&str]) -> Result<String> {
        tracing::debug!("Running git command: git {}", args.join(" "));
        
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map_err(|e| {
                tracing::error!("Failed to execute git command: {}", e);
                LauncherError::Installation(format!("Failed to run git command: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::error!("Git command failed. Stderr: {}, Stdout: {}", stderr, stdout);
            return Err(LauncherError::Installation(format!(
                "Git command failed: {}",
                stderr
            )));
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        tracing::debug!("Git command output: {}", result);
        Ok(result)
    }

    /// Check if git is available
    pub fn is_git_available() -> bool {
        Command::new("git")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get current version/commit
    pub fn get_current_version(install_path: &PathBuf) -> Result<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--short", "HEAD"])
            .current_dir(install_path)
            .output()
            .map_err(|e| LauncherError::Installation(format!("Failed to get version: {}", e)))?;

        if !output.status.success() {
            return Err(LauncherError::Installation(
                "Failed to get current version".to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

/// Default Pokemon Infinite Fusion repository
pub fn get_default_repo() -> GitHubRepo {
    GitHubRepo::new("infinitefusion", "infinitefusion-e18", "releases")
}