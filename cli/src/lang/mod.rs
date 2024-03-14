mod bash;
mod c;
mod cpp;
mod csharp;
mod dart;
mod javascript;
mod python;
mod rust;
mod shell;

pub use bash::Bash;
pub use c::C;
pub use cpp::Cpp;
pub use csharp::CSharp;
pub use dart::Dart;
pub use javascript::Javascript;
pub use python::Python;
pub use rust::Rust;
pub use shell::Shell;

use yansi::Paint as _;

use crate::fmt::Str;

#[derive(Clone, Copy, PartialEq, Eq)]
#[enum_dispatch::enum_dispatch]
pub enum Lang {
    Shell,
    Bash,
    Rust,
    Python,
    Javascript,
    C,
    Cpp,
    CSharp,
    Dart,
}

#[enum_dispatch::enum_dispatch(Lang)]
pub trait Language {
    fn as_str(&self) -> &'static str;
    fn binary(&self) -> &'static str;
    fn nix_packages(&self) -> &'static [&'static str];
    fn execute(&self, input: &str, args: impl AsRef<[String]>) -> Result<(), Str<'_>> {
        execute_interpreted(self.as_str(), self.program()?, input, args)
    }
    fn installed(&self) -> bool {
        which::which(self.binary()).is_ok()
    }
    fn program(&self) -> Result<std::process::Command, Str<'static>> {
        which::which(self.binary())
            .map(std::process::Command::new)
            .map_err(|error| exe_not_found(self.binary(), error))
            .or_else(|error| crate::nix::nix_shell(self.nix_packages(), self.binary()).ok_or(error))
    }
    #[allow(unused)]
    fn command_call<'a, D>(
        &'a self,
        command: &str,
        args: impl IntoIterator<Item = &'a D> + Clone,
    ) -> String
    where
        D: std::fmt::Display + ?Sized + 'a,
    {
        todo!("command_call not implemented for {}", self.as_str())
    }
}

fn exe_not_found(exe: impl std::fmt::Display, error: which::Error) -> Str<'static> {
    let purple = yansi::Color::BrightMagenta.bold();
    let not_found = "could not be found.\nDo you have it installed and in the PATH?\n\nRun '";
    let run = "run --commands".bright_cyan().bold();
    let for_more = "' for more information.".paint(purple);
    let error = format!(
        "{}'{exe}' {not_found}{run}{for_more}\n\nComplete error: {error}",
        "".paint(purple).linger()
    );
    Str::from(error)
}

fn installed_any(binaries: impl AsRef<[&'static str]>) -> bool {
    binaries
        .as_ref()
        .iter()
        .any(|&binary| which::which(binary).is_ok())
}

fn installed_all(binaries: impl AsRef<[&'static str]>) -> bool {
    binaries
        .as_ref()
        .iter()
        .all(|&binary| which::which(binary).is_ok())
}

fn execution_failed(exe: impl std::fmt::Display, error: impl std::fmt::Display) -> Str<'static> {
    let error = format!(
        "{}'{exe}' failed to execute command{}\n\nComplete error: {error}",
        "".bright_magenta().bold().linger(),
        "".resetting()
    );
    Str::from(error)
}

fn write_to_tmp(dir: &str, input: &str) -> Result<std::path::PathBuf, Str<'static>> {
    let to_error = |e: std::io::Error| Str::from(e.to_string());

    // Write to file to allow inheriting stdin
    let file = std::env::temp_dir().join("run/").join(dir);
    std::fs::create_dir_all(&file).map_err(to_error)?;
    let name = format!("{:x}", md5::compute(input));
    let file = file.join(name);
    std::fs::write(&file, input).map_err(to_error)?;
    Ok(file)
}

fn wait_for_child(mut child: std::process::Child) -> Result<(), Str<'static>> {
    match child.wait() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(Str::from(format!(
            "Command exited with status code {}",
            status.code().unwrap_or(-1)
        ))),
        Err(e) => Err(Str::from(format!(
            "Failed to wait for command to exit: {}",
            e
        ))),
    }
}

/// Creates a `std::process::Command` for the first program found in the PATH or in the Nix shell.
fn program_with_alternatives(
    programs: &[&'static str],
    nix_packages: &[&'static str],
) -> Result<std::process::Command, Str<'static>> {
    programs
        .iter()
        .find_map(|binary| which::which(binary).ok())
        .map(std::process::Command::new)
        .ok_or(exe_not_found(
            crate::fmt::strlist::FmtListSlice::from((&" or ", programs)),
            which::Error::CannotFindBinaryPath,
        ))
        .or_else(|error| crate::nix::nix_shell(nix_packages, programs[0]).ok_or(error))
}

/// Runs the given program with only one argument consisting in a file containing the input.
///
/// ```
/// execute_simple("python", "print('Hello')");
/// ```
/// Is equivalent to
/// ```bash
/// echo "print('Hello')" > /tmp/run/input && python /tmp/run/input
/// ```
fn execute_interpreted(
    dir: &str,
    mut program: std::process::Command,
    input: &str,
    args: impl AsRef<[String]>,
) -> Result<(), Str<'static>> {
    let args = args.as_ref();
    let file = write_to_tmp(dir, input).unwrap();
    let child =
        program.arg(file).args(args).spawn().map_err(|error| {
            execution_failed(format_args!("{:?}", program.get_program()), error)
        })?;
    wait_for_child(child)
}

/// Creates a project directory and writes the input to the main file.
///
/// If `init` is provided, it will be executed in the project directory before writing the input.
fn create_project(
    name: &str,
    init: Option<&mut std::process::Command>,
    main: impl AsRef<std::path::Path>,
    input: &str,
) -> Result<std::path::PathBuf, Str<'static>> {
    static APP_INFO: app_dirs2::AppInfo = app_dirs2::AppInfo {
        name: "runfile",
        author: "lyonsyonii",
    };
    let main = main.as_ref();

    let path = format!("cache/{name}/{:x}", md5::compute(input));
    if std::path::Path::new(&path).exists() {
        return Ok(path.into());
    }

    let Ok(path) = app_dirs2::app_dir(app_dirs2::AppDataType::UserCache, &APP_INFO, &path) else {
        return Err("Could not create project directory".into());
    };

    if let Some(init) = init {
        init.current_dir(&path)
            .output()
            .map_err(|error| execution_failed(init.get_program().to_string_lossy(), error))?;
    }

    std::fs::write(path.join(main), input)
        .map_err(|e| format!("Could not write input to {path:?}/{main:?}\nComplete error: {e}"))?;

    Ok(path)
}

/// Creates the project directory, compiles and runs the specified input.
///
/// Use in the implementation of `Language::execute`.
/// # Example
/// ```rust
/// let lang = "rust";
/// let proj_main = "src/main.rs";
/// let input = "fn main() { dbg!(3 + 3) }";
/// let args = [];
/// let init = Some(
///     self.program()?.args(["init", "--name", "runfile"])
/// );
/// let compile = self.program()?.args(["build", "--color", "always"]);
/// let run = self.program()?.args(["run", "-q", "--"]);
/// execute_compiled(lang, proj_main, input, args, init, compile, run)
/// ```
fn execute_compiled(
    lang: &str,
    proj_main: impl AsRef<std::path::Path>,
    input: &str,
    args: impl AsRef<[String]>,
    init: Option<&mut std::process::Command>,
    compile: &mut std::process::Command,
    run: &mut std::process::Command,
) -> Result<(), Str<'static>> {
    let path = create_project(lang, init, proj_main, input)?;
    std::env::set_current_dir(&path)
        .map_err(|e| format!("Could not set current directory to {path:?}\nComplete error: {e}"))?;

    let compile = compile
        .output()
        .map_err(|error| execution_failed(compile.get_program().to_string_lossy(), error))?;

    if !compile.status.success() {
        let err = String::from_utf8(compile.stderr)
            .map_err(|_| "Failed to parse command output as UTF-8")?;
        return Err(Str::from(err));
    }

    let child = run
        .args(args.as_ref())
        .spawn()
        .map_err(|error| execution_failed(run.get_program().to_string_lossy(), error))?;

    wait_for_child(child)
}

impl std::str::FromStr for Lang {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmd" | "fn" | "sh" | "shell" => Ok(Shell.into()),
            "bash" => Ok(Bash.into()),
            "rs" | "rust" => Ok(Rust.into()),
            "c" => Ok(C.into()),
            "c++" | "cpp" | "cplusplus" => Ok(Cpp.into()),
            "c#" | "cs" | "csharp" => Ok(CSharp.into()),
            "py" | "python" => Ok(Python.into()),
            "js" | "javascript" => Ok(Javascript.into()),
            "dart" => Ok(Dart.into()),
            _ => Err(s.to_owned()),
        }
    }
}

impl std::fmt::Display for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Debug for Lang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for Lang {
    fn default() -> Self {
        Shell.into()
    }
}
