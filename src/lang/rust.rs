use std::io::Write;

use colored::Colorize;

use crate::strlist::Str;

const BINARY: &str = "cargo";

pub(crate) fn installed() -> bool {
    which::which(BINARY).and(which::which("rustc")).is_ok()
}

pub(crate) fn program() -> Result<std::path::PathBuf, Str<'static>> {
    which::which(BINARY).map_err(|error| super::exe_not_found(BINARY, error))
}

pub(crate) fn execute(input: &str) -> Result<(), Str<'_>> {
    create_project(input)?;

    let out = std::process::Command::new("cargo")
        .arg("run")
        .args(["--color", "always"])
        .output()
        .map_err(|error| super::execution_failed(BINARY, error))?;

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
    let path = format!("cache/rust/{:x}", md5::compute(input));
    let Ok(path) = app_dirs2::app_dir(app_dirs2::AppDataType::UserCache, &app_info, &path) else {
        return Err("Could not create project directory".into());
    };

    let Ok(_) = std::env::set_current_dir(&path) else {
        return Err(format!("Could not set current directory to {path:?}").into());
    };

    let cargo = program()?;

    if std::fs::metadata(path.join("Cargo.toml")).is_err() {
        std::process::Command::new(cargo)
            .arg("init")
            .args(["--name", "runfile"])
            // .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|error| super::execution_failed(BINARY, error))?
            .wait()
            .map_err(|error| super::execution_failed(BINARY, error))?;
    }

    let path = path.join("src");

    let Ok(_) = std::env::set_current_dir("./src") else {
        return Err(format!("Could not set current directory to {path:?}").into());
    };

    let mut main = std::fs::File::create("main.rs")
        .map_err(|e| format!("Could not create main.rs\nComplete error: {e}"))?;

    let input = format!("fn main() {{\n{}\n}}", input);
    main.write_all(input.as_bytes())
        .map_err(|e| format!("Could not write input to main.rs\nComplete error: {e}"))?;

    Ok(())
}
