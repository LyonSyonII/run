use crate::fmt::Str;

const BINARY: &str = "dotnet";

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct CSharp;

impl super::Language for CSharp {
    fn as_str(&self) -> &'static str {
        "csharp"
    }

    fn binary(&self) -> &'static str {
        BINARY
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["dotnet-sdk"]
    }

    fn execute(&self, input: &str) -> Result<(), crate::fmt::Str<'static>> {
        create_project(|| self.program(), input)?;

        let child = self
            .program()?
            .args(["run", "--no-build"])
            .spawn()
            .map_err(|error| super::execution_failed(BINARY, error))?;

        super::wait_for_child(child)
    }
}

/// Creates a new Dotnet project in the cache directory, sets the current directory to it and writes `input` into Program.cs
fn create_project(program: impl Fn() -> Result<std::process::Command, Str<'static>>, input: &str) -> Result<(), Str<'static>> {
    let app_info = app_dirs2::AppInfo {
        name: "runfile",
        author: "lyonsyonii",
    };
    let path = format!("cache/csharp/{:x}", md5::compute(input));
    let Ok(path) = app_dirs2::app_dir(app_dirs2::AppDataType::UserCache, &app_info, &path) else {
        return Err("Could not create project directory".into());
    };

    std::env::set_current_dir(&path)
        .map_err(|e| format!("Could not set current directory to {path:?}\nComplete error: {e}"))?;

    let path = path.join("Program.cs");
    if path.exists() {
        return Ok(());
    }

    program()?
        .args(["new", "console", "-n", "runfile", "-o", "."])
        .output()
        .map_err(|e| format!("Could not create new C# project\nComplete error: {e}"))?;

    std::fs::write(path, input)
        .map_err(|e| format!("Could not write input to Program.cs\nComplete error: {e}"))?;
    
    program()?
        .arg("build")
        .output()
        .map_err(|e| format!("Could not build C# project\nComplete error: {e}"))?;
    
    Ok(())
}