use thiserror::Error;

#[derive(Debug, Error)]
pub enum UpdateCheckError {
    #[error("Failed to execute \"{0}\" command: {1}")]
    CommandExecutionError(String, String),
}
