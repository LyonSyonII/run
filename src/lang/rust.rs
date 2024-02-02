use crate::fmt::Str;

const BINARY: &str = "cargo";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rust;

impl super::Language for Rust {
    fn as_str(&self) -> &'static str {
        "rs"
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

    fn program(&self) -> Result<std::process::Command, Str<'static>> {
        which::which(BINARY)
            .map(std::process::Command::new)
            .map_err(|error| super::exe_not_found(BINARY, error))
            .or_else(|error| crate::nix::nix_shell([BINARY, "gcc"], BINARY).ok_or(error))
    }

    fn execute(&self, input: &str, args: impl AsRef<[String]>) -> Result<(), Str<'_>> {
        let input = format!("fn main() {{\n{}\n}}", input);
        super::execute_compiled(
            "rust",
            "src/main.rs",
            &input,
            args,
            Some(self.program()?.args(["init", "--name", "runfile"])),
            self.program()?.args(["build", "--color", "always"]),
            self.program()?.args(["run", "-q", "--"]),
        )
    }
}
