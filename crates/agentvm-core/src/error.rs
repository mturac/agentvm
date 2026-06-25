use thiserror::Error;

/// Errors that can occur in AgentVM operations
#[derive(Error, Debug)]
pub enum AgentVmError {
    #[error("Invalid YAML: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),

    #[error("Invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Field '{field}' is required but missing")]
    MissingField { field: String },

    #[error("Unsupported apiVersion: {version}")]
    UnsupportedVersion { version: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Memory error: {0}")]
    Memory(String),

    #[error("Skill error: {0}")]
    Skill(String),

    #[error("Platform adapter error: {0}")]
    Adapter(String),
}

/// Result type for AgentVM operations
pub type Result<T> = std::result::Result<T, AgentVmError>;
