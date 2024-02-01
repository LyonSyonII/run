use std::io::Write;

use crate::fmt::Str;

const BINARIES: &[&str] = &["gcc", "clang"];

pub(crate) fn installed() -> bool {
    BINARIES.iter().any(|&binary| which::which(binary).is_ok())
}

pub(crate) fn program() -> Result<std::process::Command, Str<'static>> {
    BINARIES
        .iter()
        .find_map(|binary| which::which(binary).ok())
        .map(std::process::Command::new)
        .ok_or(super::exe_not_found(
            "gcc or clang",
            which::Error::CannotFindBinaryPath,
        ))
        .or_else(|error| crate::nix::nix_shell(["gcc"], "gcc").ok_or(error))
}

pub(crate) fn execute(input: &str) -> Result<(), Str<'_>> {
    create_project(input)?;

    let compile = program()?
        .args(["main.c", "-o", "main"])
        .output()
        .map_err(|error| super::execution_failed("gcc/clang", error))?;

    if !compile.status.success() {
        let err = String::from_utf8(compile.stderr)
            .map_err(|_| "Failed to parse command output as UTF-8")?;
        return Err(Str::from(err));
    }

    let out = std::process::Command::new("./main")
        .output()
        .map_err(|error| super::execution_failed("gcc/clang", error))?;

    if out.status.success() {
        std::io::stdout()
            .write_all(&out.stdout)
            .map_err(|e| Str::from(e.to_string()))
    } else {
        let err =
            String::from_utf8(out.stderr).map_err(|_| "Failed to parse command output as UTF-8")?;
        Err(Str::from(err))
    }
}

/// Creates a new Rust project in the cache directory, sets the current directory to it and writes `input` into main.rs
fn create_project(input: &str) -> Result<(), Str<'static>> {
    let app_info = app_dirs2::AppInfo {
        name: "runfile",
        author: "lyonsyonii",
    };
    let path = format!("cache/c/{:x}", md5::compute(input));
    let Ok(path) = app_dirs2::app_dir(app_dirs2::AppDataType::UserCache, &app_info, &path) else {
        return Err("Could not create project directory".into());
    };

    let Ok(_) = std::env::set_current_dir(&path) else {
        return Err(format!("Could not set current directory to {path:?}").into());
    };

    let mut main = std::fs::File::create("main.c")
        .map_err(|e| format!("Could not create main.c\nComplete error: {e}"))?;

    main.write_all(input.as_bytes())
        .map_err(|e| format!("Could not write input to main.c\nComplete error: {e}"))?;

    Ok(())
}
