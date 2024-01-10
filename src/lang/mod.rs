mod bash;
mod javascript;
mod python;
mod rust;
mod shell;

use colored::Colorize as _;

use crate::strlist::Str;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Shell,
    Bash,
    Rust,
    Python,
    Javascript,
}

impl std::str::FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmd" | "fn" | "sh" | "shell" => Ok(Self::Shell),
            "bash" => Ok(Self::Bash),
            "rs" | "rust" => Ok(Self::Rust),
            "py" | "python" => Ok(Self::Python),
            "js" | "javascript" => Ok(Self::Javascript),
            _ => Err(format!("Unknown language '{s}'; expected one of [cmd, fn, sh, shell, bash, rs, rust, py, python, js, javascript]")),
        }
    }
}

impl Language {
    pub fn execute(self, input: &str) -> Result<(), Str<'_>> {
        match self {
            Language::Shell => shell::execute(input),
            Language::Bash => bash::execute(input),
            Language::Rust => rust::execute(input),
            Language::Python => python::execute(input),
            Language::Javascript => javascript::execute(input),
        }
    }
}

pub(crate) fn exe_not_found(exe: &str, error: impl std::error::Error) -> Str<'_> {
    let exe = format!("`{}`", exe).purple().bold();
    Str::from(format!(
        "{exe} {}\n\nComplete error: {error}",
        "executable could not be found.\nDo you have it installed and in the PATH?"
            .purple()
            .bold(),
    ))
}
