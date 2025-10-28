use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::error::{LauncherError, Result};

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: String,
    pub published_at: String,
    pub body: String,
    pub assets: Vec<GitHubAsset>,
    pub zipball_url: String,
    pub tarball_url: String,
}

/// GitHub release asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
    pub content_type: String,
}

/// GitHub API client
pub struct GitHubApi {
    client: Client,
    owner: String,
    repo: String,
}

impl GitHubApi {
    /// Create a new GitHub API client
    pub fn new(owner: impl Into<String>, repo: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .user_agent("Pokemon-Fusion-Launcher/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| LauncherError::GitHub(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            owner: owner.into(),
            repo: repo.into(),
        })
    }

    /// Get the latest release
    pub async fn get_latest_release(&self) -> Result<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.owner, self.repo
        );

        tracing::info!("Fetching latest release from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::GitHub(format!("Failed to fetch release: {}", e)))?;

        if !response.status().is_success() {
            return Err(LauncherError::GitHub(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let release = response
            .json::<GitHubRelease>()
            .await
            .map_err(|e| LauncherError::GitHub(format!("Failed to parse release data: {}", e)))?;

        tracing::info!("Latest release: {} ({})", release.name, release.tag_name);
        Ok(release)
    }

    /// Get a specific release by tag
    pub async fn get_release_by_tag(&self, tag: &str) -> Result<GitHubRelease> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/tags/{}",
            self.owner, self.repo, tag
        );

        tracing::info!("Fetching release {} from: {}", tag, url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::GitHub(format!("Failed to fetch release: {}", e)))?;

        if !response.status().is_success() {
            return Err(LauncherError::GitHub(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let release = response
            .json::<GitHubRelease>()
            .await
            .map_err(|e| LauncherError::GitHub(format!("Failed to parse release data: {}", e)))?;

        tracing::info!("Found release: {} ({})", release.name, release.tag_name);
        Ok(release)
    }

    /// List all releases
    pub async fn list_releases(&self) -> Result<Vec<GitHubRelease>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.owner, self.repo
        );

        tracing::info!("Fetching all releases from: {}", url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| LauncherError::GitHub(format!("Failed to fetch releases: {}", e)))?;

        if !response.status().is_success() {
            return Err(LauncherError::GitHub(format!(
                "GitHub API returned status: {}",
                response.status()
            )));
        }

        let releases = response
            .json::<Vec<GitHubRelease>>()
            .await
            .map_err(|e| LauncherError::GitHub(format!("Failed to parse releases data: {}", e)))?;

        tracing::info!("Found {} releases", releases.len());
        Ok(releases)
    }

    /// Get the download URL for a release (zipball)
    pub fn get_download_url(&self, tag: &str) -> String {
        format!(
            "https://github.com/{}/{}/archive/refs/tags/{}.zip",
            self.owner, self.repo, tag
        )
    }

    /// Get the archive download URL for a specific branch
    pub fn get_branch_archive_url(&self, branch: &str) -> String {
        format!(
            "https://github.com/{}/{}/archive/refs/heads/{}.zip",
            self.owner, self.repo, branch
        )
    }
}

/// Default Pokemon Infinite Fusion GitHub API client
pub fn get_default_api() -> Result<GitHubApi> {
    GitHubApi::new("infinitefusion", "infinitefusion-e18")
}