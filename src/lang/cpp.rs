use crate::fmt::Str;

const BINARIES: &[&str] = &["g++", "clang"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cpp;

impl super::Language for Cpp {
    fn as_str(&self) -> &'static str {
        "c++"
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

    fn program(&self) -> Result<std::process::Command, Str<'static>> {
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

    fn execute(&self, input: &str, args: impl AsRef<[String]>) -> Result<(), Str<'_>> {
        super::execute_compiled(
            "cpp",
            "main.cpp",
            input,
            args,
            None,
            self.program()?.args(["main.cpp", "-o", "main"]),
            &mut std::process::Command::new("./main"),
        )
    }
}
