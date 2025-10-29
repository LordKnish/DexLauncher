use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use crate::error::{LauncherError, Result};
use crate::core::installer::InstallProgress;
use tauri::Emitter;

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
    app_handle: tauri::AppHandle,
}

impl GitInstaller {
    /// Create a new git installer
    pub fn new(repo: GitHubRepo, app_handle: tauri::AppHandle) -> Self {
        Self { repo, app_handle }
    }

    /// Emit progress event to frontend
    fn emit_progress(&self, operation_id: &str, phase: &str, percentage: f64, message: &str) -> Result<()> {
        let progress = InstallProgress {
            operation_id: operation_id.to_string(),
            phase: phase.to_string(),
            percentage,
            message: message.to_string(),
        };

        self.app_handle
            .emit("install-progress", progress)
            .map_err(|e| LauncherError::General(format!("Failed to emit progress: {}", e)))?;

        Ok(())
    }

    /// Install or update game using git
    pub async fn install_or_update(
        &self,
        install_path: &PathBuf,
        operation_id: &str,
    ) -> Result<()> {
        // Ensure install directory exists
        std::fs::create_dir_all(install_path)?;

        // Check if .git directory exists
        let git_dir = install_path.join(".git");
        let is_existing = git_dir.exists();

        if is_existing {
            // Update existing installation
            self.update_existing(install_path, operation_id).await
        } else {
            // Fresh installation
            self.fresh_install(install_path, operation_id).await
        }
    }

    /// Perform fresh installation
    async fn fresh_install(
        &self,
        install_path: &PathBuf,
        operation_id: &str,
    ) -> Result<()> {
        self.emit_progress(operation_id, "downloading", 10.0, "Initializing git repository...")?;

        // Initialize git repository
        tracing::info!("Initializing git repository...");
        self.run_git_command(install_path, &["init", "."], operation_id, 0, 0)?;

        self.emit_progress(operation_id, "downloading", 20.0, "Adding remote origin...")?;

        // Add remote
        tracing::info!("Adding remote origin...");
        let remote_url = self.repo.clone_url();
        self.run_git_command(install_path, &["remote", "add", "origin", &remote_url], operation_id, 0, 0)?;

        self.emit_progress(operation_id, "downloading", 30.0, "Fetching from GitHub...")?;

        // Fetch with depth=1 for efficiency (this is the slow part)
        tracing::info!("Fetching from GitHub (this may take a few minutes)...");
        self.run_git_command(
            install_path,
            &["fetch", "--depth=1", "origin", &self.repo.branch, "--progress"],
            operation_id,
            30,
            80,
        )?;

        self.emit_progress(operation_id, "downloading", 80.0, "Applying files...")?;

        // Reset to fetched branch
        tracing::info!("Applying files...");
        let reset_ref = format!("origin/{}", self.repo.branch);
        self.run_git_command(install_path, &["reset", "--hard", &reset_ref], operation_id, 0, 0)?;

        self.emit_progress(operation_id, "downloading", 85.0, "Initializing submodules...")?;

        // Initialize and update submodules
        tracing::info!("Initializing submodules...");
        self.run_git_command(
            install_path,
            &["submodule", "update", "--init", "--recursive", "--progress"],
            operation_id,
            85,
            95,
        )?;

        self.emit_progress(operation_id, "downloading", 95.0, "Installation complete!")?;

        tracing::info!("Installation complete!");
        Ok(())
    }

    /// Update existing installation
    async fn update_existing(
        &self,
        install_path: &PathBuf,
        operation_id: &str,
    ) -> Result<()> {
        // Clean up shallow lock if it exists
        let shallow_lock = install_path.join(".git").join("shallow.lock");
        if shallow_lock.exists() {
            let _ = std::fs::remove_file(&shallow_lock);
        }

        self.emit_progress(operation_id, "downloading", 20.0, "Fetching updates from GitHub...")?;

        // Fetch latest changes (this is the slow part)
        tracing::info!("Fetching updates from GitHub (this may take a few minutes)...");
        self.run_git_command(
            install_path,
            &["fetch", "--depth=1", "origin", &self.repo.branch, "--progress"],
            operation_id,
            20,
            70,
        )?;

        self.emit_progress(operation_id, "downloading", 70.0, "Applying updates...")?;

        // Reset to latest
        tracing::info!("Applying updates...");
        let reset_ref = format!("origin/{}", self.repo.branch);
        self.run_git_command(install_path, &["reset", "--hard", &reset_ref], operation_id, 0, 0)?;

        self.emit_progress(operation_id, "downloading", 75.0, "Updating submodules...")?;

        // Update submodules
        tracing::info!("Updating submodules...");
        self.run_git_command(
            install_path,
            &["submodule", "update", "--init", "--recursive", "--progress"],
            operation_id,
            75,
            95,
        )?;

        self.emit_progress(operation_id, "downloading", 95.0, "Update complete!")?;

        tracing::info!("Update complete!");
        Ok(())
    }

    /// Run a git command with optional progress tracking
    fn run_git_command(
        &self,
        cwd: &PathBuf,
        args: &[&str],
        operation_id: &str,
        start_progress: u64,
        end_progress: u64,
    ) -> Result<String> {
        tracing::debug!("Running git command: git {}", args.join(" "));
        
        // If progress tracking is enabled (start != end), spawn with piped stderr
        if start_progress != end_progress {
            let mut child = Command::new("git")
                .args(args)
                .current_dir(cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .map_err(|e| {
                    tracing::error!("Failed to execute git command: {}", e);
                    LauncherError::Installation(format!("Failed to run git command: {}", e))
                })?;

            // Read stderr for progress updates
            if let Some(stderr) = child.stderr.take() {
                let mut reader = BufReader::new(stderr);
                let mut buf = Vec::with_capacity(4096);
                let mut last_percentage = 0u64;

                loop {
                    buf.clear();
                    // progress frames end with '\r'. fall back to '\n' at process end
                    let read_res = reader.read_until(b'\r', &mut buf);
                    let n = match read_res {
                        Ok(n) if n > 0 => n,
                        Ok(_) => {
                            // try to drain any final line
                            let _ = reader.read_until(b'\n', &mut buf);
                            if buf.is_empty() { break; }
                            buf.len()
                        }
                        Err(_) => break,
                    };

                    let mut s = String::from_utf8_lossy(&buf[..n]).to_string();
                    // trim carriage return and newline
                    s.retain(|c| c != '\r' && c != '\n');
                    // strip very simple ANSI sequences
                    s = s.replace("\u{001b}[K", "").replace("\u{001b}[2K", "");

                    if s.is_empty() {
                        continue;
                    }
                    tracing::debug!("Git: {}", s);

                    if let Some(percentage) = Self::parse_git_progress(&s) {
                        if percentage != last_percentage {
                            last_percentage = percentage;
                            let mapped = start_progress + ((end_progress - start_progress) * percentage / 100);

                            let status_message = if s.contains("Receiving objects") {
                                format!("Downloading files: {}%", percentage)
                            } else if s.contains("Resolving deltas") {
                                format!("Processing files: {}%", percentage)
                            } else if s.contains("Compressing objects") {
                                format!("Preparing download: {}%", percentage)
                            } else if s.contains("Counting objects") {
                                format!("Counting objects: {}%", percentage)
                            } else {
                                format!("Progress: {}%", percentage)
                            };

                            let _ = self.emit_progress(operation_id, "downloading", mapped as f64, &status_message);
                        }
                    }
                }
            }

            let output = child.wait_with_output()
                .map_err(|e| LauncherError::Installation(format!("Failed to wait for git: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::error!("Git command failed: {}", stderr);
                return Err(LauncherError::Installation(format!("Git command failed: {}", stderr)));
            }

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            // No progress tracking - use simple output capture
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
    }

    /// Parse git progress percentage from output line
    fn parse_git_progress(line: &str) -> Option<u64> {
        // find the last '%' and read contiguous digits before it
        let bytes = line.as_bytes();
        let pos = bytes.iter().rposition(|&b| b == b'%')?;
        let mut i = pos;
        // skip spaces
        while i > 0 && bytes[i - 1].is_ascii_whitespace() { i -= 1; }
        // collect digits
        let mut j = i;
        while j > 0 && bytes[j - 1].is_ascii_digit() { j -= 1; }
        if j == i { return None; }
        line[j..i].parse::<u64>().ok().filter(|n| *n <= 100)
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
