use std::io::Write;

use crate::fmt::Str;

const BINARIES: &[&str] = &["g++", "clang"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cpp;

impl super::Language for Cpp {
    fn as_str(&self) -> &'static str {
        "cpp"
    }

    fn binary(&self) -> &'static str {
        "g++"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["gcc"]
    }

    fn installed(&self) -> bool {
        BINARIES.iter().any(|&binary| which::which(binary).is_ok())
    }

    fn program(&self) -> Result<std::process::Command, Str<'_>> {
        BINARIES
            .iter()
            .find_map(|binary| which::which(binary).ok())
            .map(std::process::Command::new)
            .ok_or(super::exe_not_found(
                "g++ or clang",
                which::Error::CannotFindBinaryPath,
            ))
            .or_else(|error| crate::nix::nix_shell(["gcc"], "g++").ok_or(error))
    }

    fn execute(&self,input: &str) -> Result<(),Str<'_>> {
        create_project(input)?;
    
        let compile = self.program()?
            .args(["main.cpp", "-o", "main"])
            .output()
            .map_err(|error| super::execution_failed("g++/clang", error))?;
        
        if !compile.status.success() {
            let err = String::from_utf8(compile.stderr)
                .map_err(|_| "Failed to parse command output as UTF-8")?;
            return Err(Str::from(err));
        }
        
        let child = std::process::Command::new("./main")
            .spawn()
            .map_err(|error| super::execution_failed("g++/clang", error))?;

        super::wait_for_child(child)
    }
}

/// Creates a new Rust project in the cache directory, sets the current directory to it and writes `input` into main.rs
fn create_project(input: &str) -> Result<(), Str<'static>> {
    let app_info = app_dirs2::AppInfo {
        name: "runfile",
        author: "lyonsyonii",
    };
    let path = format!("cache/cpp/{:x}", md5::compute(input));
    let Ok(path) = app_dirs2::app_dir(app_dirs2::AppDataType::UserCache, &app_info, &path) else {
        return Err("Could not create project directory".into());
    };

    std::fs::write(path.join("main.cpp"), input)
        .map_err(|e| format!("Could not write input to main.cpp\nComplete error: {e}"))?;

    Ok(())
}
