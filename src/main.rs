use ariadne::{sources, Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use chumsky::Parser as _;
use command::Command;
pub use std::format as fmt;

mod command;
mod lang;
mod parser;
mod runfile;
mod strlist;
mod utils;

fn main() -> std::io::Result<()> {
    let (file, input) = get_file();
    let runfile = match parser::runfile().parse(&input).into_result() {
        Ok(r) => r,
        Err(errors) => {
            let mut colors = ColorGenerator::new();

            for e in errors {
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
            std::process::exit(1);
        }
    };

    let current_exe = std::env::current_exe()?;
    let exe_name = current_exe
        .file_name()
        .unwrap_or(std::ffi::OsStr::new("run"));
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    runfile
        .run((" ", [exe_name.to_string_lossy()]), args)
        .unwrap();

    Ok(())
}

fn get_file() -> (&'static str, String) {
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
            return (file, contents);
        }
    }
    eprintln!("Could not find runfile");
    eprintln!("Possible file names: {files:?}");
    eprintln!("See `run --help` for more information");
    std::process::exit(1);
}
