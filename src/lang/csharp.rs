#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct CSharp;

impl super::Language for CSharp {
    fn as_str(&self) -> &'static str {
        "csharp"
    }

    fn binary(&self) -> &'static str {
        "dotnet"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["dotnet-sdk"]
    }

    fn execute(
        &self,
        input: &str,
        args: impl AsRef<[String]>,
    ) -> Result<(), crate::fmt::Str<'static>> {
        super::execute_compiled(
            "csharp",
            "Program.cs",
            input,
            args,
            Some(
                self.program()?
                    .args(["new", "console", "-n", "runfile", "-o", "."]),
            ),
            self.program()?.arg("build"),
            self.program()?.args(["run", "--no-build"]),
        )
    }
}
