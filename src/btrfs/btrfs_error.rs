use thiserror::Error;
use crate::command_runner::CommandError;

#[derive(Debug, Error)]
pub enum BtrfsError {
    #[error(transparent)]
    CommandExecutionError(#[from] CommandError),
    #[error(transparent)]
    SpawnError(#[from] tokio::task::JoinError),
    #[error("Unable to run btrfs scrub: {0}")]
    ScrubError(String),
    #[error("Unable to parse btrfs output: {0}")]
    FormatError(String),
}
