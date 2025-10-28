use std::fmt;

/// Custom error types for the launcher
#[derive(Debug)]
pub enum LauncherError {
    /// Database errors
    Database(String),
    /// Network/download errors
    Network(String),
    /// File system errors
    FileSystem(String),
    /// Installation errors
    Installation(String),
    /// Verification errors
    Verification(String),
    /// Configuration errors
    Config(String),
    /// GitHub API errors
    GitHub(String),
    /// General errors
    General(String),
}

impl fmt::Display for LauncherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LauncherError::Database(msg) => write!(f, "Database error: {}", msg),
            LauncherError::Network(msg) => write!(f, "Network error: {}", msg),
            LauncherError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            LauncherError::Installation(msg) => write!(f, "Installation error: {}", msg),
            LauncherError::Verification(msg) => write!(f, "Verification error: {}", msg),
            LauncherError::Config(msg) => write!(f, "Configuration error: {}", msg),
            LauncherError::GitHub(msg) => write!(f, "GitHub API error: {}", msg),
            LauncherError::General(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for LauncherError {}

// Implement From traits for common error types
impl From<rusqlite::Error> for LauncherError {
    fn from(err: rusqlite::Error) -> Self {
        LauncherError::Database(err.to_string())
    }
}

impl From<r2d2::Error> for LauncherError {
    fn from(err: r2d2::Error) -> Self {
        LauncherError::Database(format!("Connection pool error: {}", err))
    }
}

impl From<reqwest::Error> for LauncherError {
    fn from(err: reqwest::Error) -> Self {
        LauncherError::Network(err.to_string())
    }
}

impl From<std::io::Error> for LauncherError {
    fn from(err: std::io::Error) -> Self {
        LauncherError::FileSystem(err.to_string())
    }
}

impl From<zip::result::ZipError> for LauncherError {
    fn from(err: zip::result::ZipError) -> Self {
        LauncherError::FileSystem(format!("ZIP error: {}", err))
    }
}

impl From<serde_json::Error> for LauncherError {
    fn from(err: serde_json::Error) -> Self {
        LauncherError::Config(format!("JSON error: {}", err))
    }
}

// Convert LauncherError to String for Tauri commands
impl From<LauncherError> for String {
    fn from(err: LauncherError) -> Self {
        err.to_string()
    }
}

/// Result type alias for launcher operations
pub type Result<T> = std::result::Result<T, LauncherError>;