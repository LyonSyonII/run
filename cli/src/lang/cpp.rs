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
        super::installed_any(BINARIES)
    }

    fn program(&self) -> Result<std::process::Command, Str<'static>> {
        super::program_with_alternatives(BINARIES, self.nix_packages())
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
