mod bash;
mod c;
mod cpp;
mod javascript;
mod python;
mod rust;
mod shell;

use yansi::Paint as _;

use crate::fmt::Str;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Language {
    #[default]
    Shell,
    Bash,
    Rust,
    Python,
    Javascript,
    C,
    Cpp,
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
            Self::Cpp => "c++",
        }
    }

    pub fn execute(self, input: &str) -> Result<(), Str<'_>> {
        match self {
            Language::Shell => shell::execute(input),
            Language::Bash => bash::execute(input),
            Language::Rust => rust::execute(input),
            Language::C => c::execute(input),
            Language::Cpp => cpp::execute(input),
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
            Language::Cpp => cpp::installed(),
            Language::Python => python::installed(),
            Language::Javascript => javascript::installed(),
        }
    }
}

pub(crate) fn exe_not_found(exe: &str, error: which::Error) -> Str<'_> {
    let purple = yansi::Color::BrightMagenta.bold();
    let not_found =
        "executable could not be found.\nDo you have it installed and in the PATH?\n\nRun '";
    let run = "run --commands".bright_cyan().bold();
    let for_more = "' for more information.".paint(purple);
    let error = format!(
        "{}'{exe}' {not_found}{run}{for_more}\n\nComplete error: {error}",
        "".paint(purple).linger()
    );
    Str::from(error)
}

pub(crate) fn execution_failed(exe: &str, error: impl std::fmt::Display) -> Str<'_> {
    let error = format!(
        "{}'{exe}' failed to execute command{}\n\nComplete error: {error}",
        "".bright_magenta().bold().linger(),
        "".clear()
    );
    Str::from(error)
}

impl std::str::FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmd" | "fn" | "sh" | "shell" => Ok(Self::Shell),
            "bash" => Ok(Self::Bash),
            "rs" | "rust" => Ok(Self::Rust),
            "c" => Ok(Self::C),
            "c++" | "cpp" => Ok(Self::Cpp),
            "py" | "python" => Ok(Self::Python),
            "js" | "javascript" => Ok(Self::Javascript),
            _ => Err(s.to_owned()),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
