use ariadne::{Color, ColorGenerator};
use colored::Colorize as _;
use strlist::Str;
use utils::OptionExt as _;

use crate::error::Error;

pub type HashMap<K, V> = indexmap::IndexMap<K, V, xxhash_rust::xxh3::Xxh3Builder>;

mod clap;
mod command;
mod error;
mod lang;
mod nix;
mod parsing;
mod runfile;
mod strlist;
mod utils;

fn main() -> std::io::Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();

    if args.first().is_some_and_oneof(["-h", "--help"]) {
        print_help();
        return Ok(());
    };

    if args.first().is_some_and_oneof(["--print-complete"]) {
        crate::clap::print_completion();
        return Ok(());
    }

    let (file, input) = get_file(&mut args);
    let dot = std::path::Path::new(".");
    let path = if file == "stdin" {
        dot
    } else {
        std::path::Path::new(file.as_ref())
            .parent()
            .map(|mut p| {
                if p == std::path::Path::new("") {
                    p = dot
                }
                p
            })
            .unwrap_or(dot)
    };

    let runfile = match parsing::parser::runfile(&input, path) {
        Ok(r) => match r {
            Ok(r) => r,
            Err(errors) => {
                print_errors(errors, file, &input)?;
                std::process::exit(1);
            }
        },
        Err(e) => {
            let start = e.location.offset;
            let msg = format!("Expected {}", e.expected);
            Error::ariadne(msg, start, start, file, &input, Color::Magenta)?;
            std::process::exit(1);
        }
    };

    runfile.run((" ", [get_current_exe()?]), &args).unwrap();

    Ok(())
}

fn print_help() {
    println!(
        "{}",
        "Run commands in the languages you love!\n".dimmed().bold()
    );
    println!("Runs a runfile in the current directory");
    println!("Possible names: [run, runfile] or any ending in '.run'\n");
    println!("Commands can be written in any language supported by runfile");
    println!(
        "If the language's compiler is not installed, 'run' will try to use nix-shell instead\n"
    );
    println!(
        "See {} for more information\n",
        "https://github.com/lyonsyonii/run".bold()
    );
    println!(
        "{} {} {}\n",
        "Usage:".bright_green().bold(),
        "run".bright_cyan().bold(),
        "[COMMAND] [ARGS...]".cyan()
    );
    println!("{}", "Options:".bright_green().bold());
    println!(
        "  {}, {} {}\tRuns the specified file instead of searching for a runfile",
        "-f".bright_cyan().bold(),
        "--file".bright_cyan().bold(),
        "<FILE>".cyan()
    );
    println!(
        "  {}, {}\tPrints available commands in the runfile",
        "-c".bright_cyan().bold(),
        "--commands".bright_cyan().bold()
    );
    println!(
        "      {}\tPrints the completion script for the current shell",
        "--print-complete".bright_cyan().bold()
    );
    println!(
        "      {}\t\tEnables reading the runfile from stdin",
        "--stdin".bright_cyan().bold()
    );
    println!(
        "  {}, {}\t\tPrints help information",
        "-h".bright_cyan().bold(),
        "--help".bright_cyan().bold()
    );
}

fn print_errors(
    errors: impl AsRef<[crate::error::Error]>,
    file: impl AsRef<str>,
    input: impl AsRef<str>,
) -> std::io::Result<()> {
    let file = file.as_ref();
    let input = input.as_ref();
    let errors = errors.as_ref();
    let mut colors = ColorGenerator::new();

    for e in errors {
        e.eprint(file, input, colors.next())?;
    }

    Ok(())
}

fn get_file(args: &mut Vec<String>) -> (Str<'static>, String) {
    let first = args.first();

    if let (Some(file), Some("--stdin")) = (read_pipe::read_pipe(), first.map(|s| s.as_str())) {
        // Remove --stdin from args
        args.remove(0);
        return ("stdin".into(), file);
    }
    
    if first.is_some_and_oneof(["-f", "--file"]) {
        let file = args.get(1);
        if let Some(file) = file {
            if let Ok(contents) = std::fs::read_to_string(file) {
                let file = file.to_owned().into();
                // Remove -f and the file name from args
                args.drain(..=1);
                return (file, contents);
            }

            let err = format!("Error: Could not read file '{}'", file)
                .bright_red()
                .bold();
            eprintln!("{}", err);
            std::process::exit(1);
        }

        eprintln!(
            "{}\n{} {} {}",
            "Error: No file specified".bright_red().bold(),
            "Usage:".bright_green().bold(),
            "run --file <FILE>".bright_cyan().bold(),
            "[COMMAND] [ARGS...]".cyan()
        );
        std::process::exit(1);
    }

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
            let contents = std::fs::read_to_string(path);

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
