use std::io::Write;
use crate::strlist::Str;

const BINARY: &str = "python";

pub(crate) fn installed() -> bool {
    which::which(BINARY).is_ok() || crate::nix::is_nix()
}

pub(crate) fn program() -> Result<std::process::Command, Str<'static>> {
    which::which(BINARY)
        .map(std::process::Command::new)
        .map_err(|error| super::exe_not_found(BINARY, error))
        .or_else(|error| crate::nix::nix_shell(["python3Minimal"], "python").ok_or(error))
}

pub(crate) fn execute(input: &str) -> Result<(), Str<'_>> {
    let to_error = |e: std::io::Error| Str::from(e.to_string());

    let mut child = program()?
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|error| super::execution_failed(BINARY, error))?;

    child
        .stdin
        .take()
        .ok_or("Failed to open stdin")?
        .write_all(input.as_bytes())
        .map_err(to_error)?;

    let out = child.wait_with_output().map_err(to_error)?;

    if out.status.success() {
        let out = std::str::from_utf8(&out.stdout)
            .map_err(|_| "Failed to parse command output as UTF-8")?;
        print!("{out}");
        Ok(())
    } else {
        let err =
            String::from_utf8(out.stderr).map_err(|_| "Failed to parse command output as UTF-8")?;
        Err(Str::from(err))
    }
}
