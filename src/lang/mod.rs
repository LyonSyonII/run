mod bash;
mod c;
mod cpp;
mod javascript;
mod python;
mod rust;
mod shell;

use yansi::Paint as _;

use crate::fmt::Str;

// TODO: Use enum_dispatch or something similar to avoid boilerplate

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    Shell,
    Bash,
    Rust,
    Python,
    Javascript,
    C,
    Cpp,
}

impl Lang {
    pub fn as_str(self) -> &'static str {
        self.as_language().as_str()
    }

    pub fn execute(self, input: &str) -> Result<(), Str<'_>> {
        self.as_language().execute(input)
    }

    pub fn installed(self) -> bool {
        crate::nix::is_nix() || self.as_language().installed()
    }

    pub fn as_language(self) -> &'static dyn Language {
        match self {
            Lang::Shell => shell::Shell,
            Lang::Bash => bash::Bash,
            Lang::Rust => rust::Rust,
            Lang::C => c::C,
            Lang::Cpp => cpp::Cpp,
            Lang::Python => &python::Python,
            Lang::Javascript => &javascript::Javascript,
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

impl std::str::FromStr for Lang {
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

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

trait Language {
    fn execute(&self, input: &str) -> Result<(), Str<'_>>;
    fn installed(&self) -> bool;
    fn program(&self) -> Result<std::process::Command, Str<'_>>;
    fn as_str(&self) -> &'static str;
}