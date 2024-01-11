use ariadne::{sources, Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use chumsky::{error::Rich, Parser as _};
use colored::Colorize as _;
pub use std::format as fmt;
use strlist::Str;
use utils::OptionExt as _;

mod command;
mod lang;
mod parser;
mod runfile;
mod strlist;
mod utils;

fn main() -> std::io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.first().is_some_and_oneof(["-h", "--help"]) {
        print_help();
        return Ok(());
    };

    let (file, input) = get_file();
    let runfile = match parser::runfile().parse(&input).into_result() {
        Ok(r) => r,
        Err(errors) => {
            print_errors(errors, file, &input)?;
            std::process::exit(1);
        }
    };

    // dbg!(&runfile);

    runfile.run((" ", [get_current_exe()?]), &args).unwrap();

    Ok(())
}

fn print_help() {
    println!("Runs a runfile in the current directory");
    println!("Possible runfile names: [run, runfile] or any ending in '.run'\n");
    println!(
        "{} {} {}\n",
        "Usage:".bright_green().bold(),
        "run".bright_cyan().bold(),
        "[COMMAND] [ARGS...]".cyan()
    );
    println!("{}", "Options:".bright_green().bold());
    println!(
        "  {}, {}\t\tPrints help information",
        "-h".bright_cyan().bold(),
        "--help".bright_cyan().bold()
    );
    println!(
        "  {}, {}\tPrints available commands in the runfile",
        "-c".bright_cyan().bold(),
        "--commands".bright_cyan().bold()
    );
}

fn print_errors<'a>(
    errors: impl AsRef<[Rich<'a, char>]>,
    file: impl AsRef<str>,
    input: impl AsRef<str>,
) -> std::io::Result<()> {
    let errors = errors.as_ref();
    let mut colors = ColorGenerator::new();

    for e in errors {
        let file = file.as_ref();
        ariadne::Report::build(ReportKind::Error, file, e.span().start)
            .with_message(e.to_string())
            .with_label(
                Label::new((file, e.span().into_range()))
                    .with_message(e.reason().fg(Color::Red))
                    .with_color(colors.next()),
            )
            .finish()
            .eprint((file, Source::from(&input)))?;
    }

    Ok(())
}

fn get_file() -> (Str<'static>, String) {
    let files = [
        "runfile",
        "run",
        "Runfile",
        "Run",
        "runfile.run",
        "run.run",
        "Runfile.run",
        "Run.run",
    ];
    for file in files {
        if let Ok(contents) = std::fs::read_to_string(file) {
            return (file.into(), contents);
        }
    }

    let files = match std::fs::read_dir(".") {
        Ok(f) => f,
        Err(e) => {
            eprintln!(
                "{} {}",
                "Error:".bright_red().bold(),
                e.to_string().bright_red().bold()
            );
            std::process::exit(1);
        }
    };

    for file in files.flatten() {
        let path = file.path();
        if path.extension() == Some(std::ffi::OsStr::new("run")) {
            let name = path
                .file_name()
                .map(|p| p.to_string_lossy().to_string().into());
            let contents = std::fs::read_to_string(file.path());

            if let (Some(name), Ok(contents)) = (name, contents) {
                return (name, contents);
            }
        }
    }
    eprintln!("{}", "Error: Could not find runfile".bold().bright_red());
    eprintln!(
        "Possible file names: [{}, {}] or any ending in {}",
        "run".bright_purple().bold(),
        "runfile".bright_purple().bold(),
        ".run".bright_purple().bold()
    );
    eprintln!(
        "See '{} {}' for more information",
        "run".bright_cyan().bold(),
        "--help".bright_cyan().bold()
    );
    std::process::exit(1);
}

fn get_current_exe() -> std::io::Result<String> {
    let current_exe = std::env::current_exe()?;
    let exe_name = current_exe
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("run"));
    Ok(exe_name.to_string_lossy().to_string())
}
