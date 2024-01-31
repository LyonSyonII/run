use std::env::temp_dir;

use crate::strlist::Str;

const BINARY: &str = "sh";

pub(crate) fn installed() -> bool {
    which::which(BINARY).is_ok()
}

pub(crate) fn program() -> Result<std::path::PathBuf, Str<'static>> {
    which::which(BINARY).map_err(|error| super::exe_not_found(BINARY, error))
}

pub(crate) fn execute(input: &str) -> Result<(), Str<'_>> {
    let to_error = |e: std::io::Error| Str::from(e.to_string());

    // Write to file to allow inheriting stdin
    let file = temp_dir().join("run/shell");
    std::fs::create_dir_all(&file).map_err(to_error)?;
    let file = file.join("input.sh");
    std::fs::write(&file, input).map_err(to_error)?;

    let mut child = std::process::Command::new(program()?)
        .arg(file)
        .spawn()
        .map_err(|error| super::execution_failed(BINARY, error))?;

    match child.wait() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(Str::from(format!(
            "Command exited with status code {}",
            status.code().unwrap_or(-1)
        ))),
        Err(e) => Err(Str::from(format!(
            "Failed to wait for command to exit: {}",
            e
        ))),
    }
}
