use thiserror::Error;

#[derive(Debug, Error)]
pub enum SmartCheckError {
    #[error("Failed to execute \"{0}\" command: {1}")]
    CommandExecutionError(String, String),
    #[error("Failed to start async task: {0}")]
    SpawnError(String),
}
