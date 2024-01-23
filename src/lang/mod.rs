mod bash;
mod javascript;
mod python;
mod rust;
mod shell;
mod c;

use colored::Colorize as _;

use crate::strlist::Str;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Shell,
    Bash,
    Rust,
    Python,
    Javascript,
    C,
}

impl Language {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Shell => "sh",
            Self::Bash => "bash",
            Self::Rust => "rs",
            Self::Python => "py",
            Self::Javascript => "js",
            Self::C => "c",
        }
    }

    pub fn execute(self, input: &str) -> Result<(), Str<'_>> {
        match self {
            Language::Shell => shell::execute(input),
            Language::Bash => bash::execute(input),
            Language::Rust => rust::execute(input),
            Language::C => c::execute(input),
            Language::Python => python::execute(input),
            Language::Javascript => javascript::execute(input),
        }
    }

    pub fn installed(self) -> bool {
        match self {
            Language::Shell => shell::installed(),
            Language::Bash => bash::installed(),
            Language::Rust => rust::installed(),
            Language::C => c::installed(),
            Language::Python => python::installed(),
            Language::Javascript => javascript::installed(),
        }
    }
}

pub(crate) fn exe_not_found(exe: &str, error: which::Error) -> Str<'_> {
    let exe = format!("'{}'", exe).bright_purple().bold();
    Str::from(format!(
        "{exe} {}{}{}\n\nComplete error: {error}",
        "executable could not be found.\nDo you have it installed and in the PATH?\n\nRun '"
            .bright_purple()
            .bold(),
        "run --commands".bright_cyan().bold(),
        "' for more information.".bright_purple().bold(),
    ))
}

pub(crate) fn execution_failed(exe: &str, error: impl std::fmt::Display) -> Str<'_> {
    let exe = format!("'{}'", exe).bright_purple().bold();
    Str::from(format!(
        "{exe} {}\n\nComplete error: {error}",
        "failed to execute command".bright_purple().bold()
    ))
}

impl std::str::FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmd" | "fn" | "sh" | "shell" => Ok(Self::Shell),
            "bash" => Ok(Self::Bash),
            "rs" | "rust" => Ok(Self::Rust),
            "c" => Ok(Self::C),
            "py" | "python" => Ok(Self::Python),
            "js" | "javascript" => Ok(Self::Javascript),
            _ => Err(format!("Unknown language '{s}'; expected one of [cmd, fn, sh, shell, bash, rs, rust, py, python, js, javascript]")),
        }
    }
}
