use std::io::Write;

use crate::fmt::Str;

const BINARY: &str = "cargo";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rust;

impl super::Language for Rust {
    fn as_str(&self) -> &'static str {
        "rust"
    }

    fn binary(&self) -> &'static str {
        BINARY
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["rustc"]
    }

    fn installed(&self) -> bool {
        which::which(BINARY).and(which::which("rustc")).is_ok()
    }

    fn program(&self) -> Result<std::process::Command, Str<'_>> {
        which::which(BINARY)
            .map(std::process::Command::new)
            .map_err(|error| super::exe_not_found(BINARY, error))
            .or_else(|error| crate::nix::nix_shell([BINARY, "gcc"], BINARY).ok_or(error))
    }

    fn execute(&self, input: &str) -> Result<(), Str<'_>> {
        create_project(self.program()?, input)?;

        let compile = self
            .program()?
            .args(["build", "--color", "always"])
            .output()
            .map_err(|error| super::execution_failed(BINARY, error))?;
        
        if !compile.status.success() {
            let err = String::from_utf8(compile.stderr)
                .map_err(|_| "Failed to parse command output as UTF-8")?;
            return Err(Str::from(err));
        }

        let child = self.program()?
            .args(["run", "-q"])
            .spawn()
            .map_err(|error| super::execution_failed(BINARY, error))?;

        super::wait_for_child(child)
    }
}

/// Creates a new Rust project in the cache directory, sets the current directory to it and writes `input` into main.rs
fn create_project(mut program: std::process::Command, input: &str) -> Result<(), Str<'static>> {
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

    if std::fs::metadata(path.join("Cargo.toml")).is_err() {
        program
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
