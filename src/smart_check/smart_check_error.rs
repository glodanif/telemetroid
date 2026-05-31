use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmartCheckError {
    #[error("Failed to execute \"{0}\" command: {1}")]
    CommandExecutionError(String, String),
    #[error("Failed to start async task: {0}")]
    SpawnError(String),
    #[error("Unable to scan for drives: {0}")]
    ScanError(String),
    #[error("Unable to parse smartctl output: {0}")]
    FormatError(String),
}
