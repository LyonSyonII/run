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
        &[BINARY, "gcc"]
    }

    fn installed(&self) -> bool {
        super::installed_all([BINARY, "rustc"])
    }

    fn command_call<'a, D>(&'a self, args: impl IntoIterator<Item = &'a D> + Clone) -> String
    where
        D: std::fmt::Display + ?Sized + 'a,
    {
        let args = crate::fmt::strlist::FmtIter::new(",", args);
        format!("::std::process::Command::new(\"run\").args({args}).output().unwrap()")
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
