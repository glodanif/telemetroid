use std::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("failed to run `{cmd}`: {source}")]
    Spawn { cmd: String, #[source] source: std::io::Error },

    #[error("`{cmd}` terminated by signal: {detail}")]
    Signal { cmd: String, detail: String },

    #[error("`{cmd}` failed (exit {code}): {detail}")]
    Exit { cmd: String, code: i32, detail: String },
}

pub fn run_command(
    command: &str,
    arguments: &[&str],
    is_failure: impl Fn(i32) -> bool,
) -> Result<Vec<u8>, CommandError> {
    let cmd = format!("{} {}", command, arguments.join(" "));
    log::debug!("Running command: {}", cmd);

    let output = Command::new(command)
        .args(arguments)
        .output()
        .map_err(|source| CommandError::Spawn { cmd: cmd.clone(), source })?;

    let Some(code) = output.status.code() else {
        let detail = stderr_or(&output.stderr, "process terminated by signal");
        return Err(CommandError::Signal { cmd, detail });
    };

    if is_failure(code) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = stderr_or(&output.stderr, &stdout);
        return Err(CommandError::Exit { cmd, code, detail });
    }

    Ok(output.stdout)
}

fn stderr_or(stderr: &[u8], fallback: &str) -> String {
    let s = String::from_utf8_lossy(stderr);
    if s.trim().is_empty() { fallback.to_string() } else { s.into_owned() }
}
