use std::io::Write;

use crate::fmt::Str;

const BINARIES: &[&str] = &["gcc", "clang"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct C;

impl super::Language for C {
    fn as_str(&self) -> &'static str {
        "c"
    }

    fn binary(&self) -> &'static str {
        "gcc"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["gcc"]
    }

    fn installed(&self) -> bool {
        BINARIES.iter().any(|&binary| which::which(binary).is_ok())
    }

    fn program(&self) -> Result<std::process::Command, Str<'static>> {
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

    fn execute(&self, input: &str) -> Result<(), Str<'_>> {
        super::execute_compiled(
            "c",
            "main.c",
            input,
            None,
            self.program()?.args(["main.c", "-o", "main"]),
            &mut std::process::Command::new("./main"),
        )
    }
}
