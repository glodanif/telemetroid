use thiserror::Error;
use crate::command_runner::CommandError;

#[derive(Debug, Error)]
pub enum SmartCheckError {
    #[error(transparent)]
    CommandExecutionError(#[from] CommandError),
    #[error("Unable to scan for drives: {0}")]
    ScanError(String),
    #[error("Unable to parse smartctl output: {0}")]
    FormatError(String),
}
