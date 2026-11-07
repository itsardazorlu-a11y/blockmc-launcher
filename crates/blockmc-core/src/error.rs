#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("invalid instance id")]
    InvalidId,
    #[error("instance name cannot be empty")]
    EmptyName,
    #[error("instance already exists")]
    AlreadyExists,
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    Unsupported(String),
    #[error("network error: {0}")]
    Net(#[from] reqwest::Error),
    #[error("download blocked: {0} is not a trusted Modrinth address")]
    UntrustedUrl(String),
    #[error("the downloaded file did not match its checksum, so it was discarded")]
    HashMismatch,
    #[error("Modrinth gave no checksum for this file, so it was not installed")]
    HashMissing,
    #[error("no version of {0} is compatible with this instance")]
    NoCompatibleVersion(String),
    #[error("already installed")]
    AlreadyInstalled,
    #[error("unsafe file name: {0}")]
    UnsafeFilename(String),
    #[error("invalid project id")]
    InvalidProject,
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid data file: {0}")]
    Json(#[from] serde_json::Error),
}
