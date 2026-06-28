use thiserror::Error;
use crate::command_runner::CommandError;

#[derive(Debug, Error)]
pub enum SmartCheckError {
    #[error(transparent)]
    CommandExecutionError(#[from] CommandError),
    #[error(transparent)]
    SpawnError(#[from] tokio::task::JoinError),
    #[error("Unable to scan for drives: {0}")]
    ScanError(String),
    #[error("Unable to parse smartctl output: {0}")]
    FormatError(String),
    #[error("self-test on {0} did not complete: {1}")]
    TestIncomplete(String, String),
    #[error("a SMART self-test is already running on the system")]
    AlreadyRunningError(),
}
