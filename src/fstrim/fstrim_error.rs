use thiserror::Error;
use crate::command_runner::CommandError;

#[derive(Debug, Error)]
pub enum FstrimError {
    #[error(transparent)]
    CommandExecutionError(#[from] CommandError),
    #[error(transparent)]
    SpawnError(#[from] tokio::task::JoinError),
    #[error("Unable to parse fstrim output: {0}")]
    FormatError(String),
}
