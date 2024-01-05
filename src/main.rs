use std::{collections::HashMap, ops::Deref};
use runner::Command;

mod parser;
mod runner;

fn main() -> std::io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args
        .iter()
        .any(|arg| matches!(arg.as_str(), "-h" | "--help"))
    {
        println!("run: Runs a script.sh file in the current directory.");
        return Ok(());
    }
    
    let runfile = get_file();
    let runfile = parser::runfile::parse(runfile.deref()).expect("could not parse runfile");

    match args.first().and_then(|c| runfile.commands.get(c.as_str())) {
        Some(cmd) => cmd.run(args.get(1..).unwrap_or_default())?,
        None => {
            let cmd = runfile.commands.get("default").goodbye(format!(
                "could not find default command\navailable commands: {:?}",
                runfile.commands.keys()
            ));
            cmd.run(args)?;
        }
    }

    Ok(())
}

fn get_file() -> String {
    let files = ["runfile", "run", "Runfile", "Run"];
    for file in files {
        if let Ok(file) = std::fs::read_to_string(file) {
            return file;
        } 
    }
    eprintln!("run: could not find runfile");
    eprintln!("run: available files: {files:?}");
    std::process::exit(1);
}

pub struct Runfile<'i> {
    commands: HashMap<&'i str, Command<'i>>,
}

trait Goodbye<T> {
    fn goodbye(self, msg: impl AsRef<str>) -> T;
}

impl<T> Goodbye<T> for Option<T> {
    fn goodbye(self, msg: impl AsRef<str>) -> T {
        self.unwrap_or_else(|| {
            println!("run: {}", msg.as_ref());
            std::process::exit(1)
        })
    }
}

impl<T, E> Goodbye<T> for Result<T, E> {
    fn goodbye(self, msg: impl AsRef<str>) -> T {
        self.unwrap_or_else(|_| {
            println!("run: {}", msg.as_ref());
            std::process::exit(1)
        })
    }
}
