use crate::strlist::Str;

const BINARY: &str = "cargo";

pub(crate) fn installed() -> bool {
    which::which(BINARY).is_ok()
}

pub(crate) fn program() -> Result<std::path::PathBuf, Str<'static>> {
    which::which(BINARY).map_err(|error| super::exe_not_found(BINARY, error))
}

pub(crate) fn execute(input: &str) -> Result<(), Str<'_>> {
    let to_error = |e: std::io::Error| Str::from(e.to_string());
    
    let mut child = std::process::Command::new(program()?)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| super::execution_failed(BINARY, error))?;

    todo!()
}