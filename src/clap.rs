use std::io::Write as _;

use crate::runfile::Runfile;
use clap::{arg, Command};
use clap_complete::shells::Shell;

pub fn print_completion() {
    // let app = build_cli(runfile);
    // gen_completion(get_shell(), app, &mut std::io::stdout());
    let mut cmd = clap::Command::new("run").args([
        clap::Arg::new("help")
            .short('h')
            .long("help")
            .help("Prints help information"),
        clap::Arg::new("commands")
            .short('c')
            .long("commands")
            .help("Prints available commands in the runfile"),
        clap::Arg::new("print-complete")
            .long("print-complete")
            .help("Prints the completion script for the current shell"),
        clap::Arg::new("file")
            .short('f')
            .long("file")
            .value_name("FILE")
            .help("Runs the specified file instead of searching for a runfile"),
    ]);
    clap_complete::generate(get_shell(), &mut cmd, "run", &mut std::io::stdout());
}

fn get_shell() -> Shell {
    let shell = std::process::Command::new("ps")
        .arg("-o")
        .arg("comm=")
        .output()
        .unwrap()
        .stdout;
    std::str::from_utf8(&shell)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .parse::<Shell>()
        .unwrap_or(Shell::Bash)
}

// UNUSED at the moment, will be used if 'run' is migrated to 'clap'
#[allow(dead_code)]
mod unused {
    use super::*;
    fn build_cli(runfile: &Runfile<'_>) -> clap::Command {
        let commands = runfile.commands.iter().map(|(name, c)| {
            clap::Command::new(name.to_string())
                .about(c.doc_raw().to_owned())
                .args(
                    c.args()
                        .iter()
                        .map(|a| clap::Arg::new(a.to_string()).required(true)),
                )
        });

        clap::Command::new("run")
            .args([
                arg!(-h --help "Prints help information"),
                arg!(-c --commands "Prints available commands in the runfile"),
                arg!(-f --file <FILE> "Runs the specified file instead of searching for a runfile"),
            ])
            .subcommands(commands)
    }

    pub fn gen_completion(shell: Shell, mut app: Command, to: &mut impl std::io::Write) {
        clap_complete::generate(shell, &mut app, "run", to);
    }

    pub fn write_completions(runfile: &Runfile<'_>) {
        let app = build_cli(runfile);

        let shell = get_shell();

        let mut out = Vec::<u8>::new();
        gen_completion(shell, app, &mut out);
        println!("{}", std::str::from_utf8(&out).unwrap());

        let mut child = std::process::Command::new("complete")
            .arg("/dev/stdin")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .unwrap()
            .stdin
            .unwrap();
        child.write_all(&out).unwrap();
        // let mut zsh = std::fs::File::create("run.zsh").unwrap();
        // let mut bash = std::fs::File::create("run.bash").unwrap();
        // let mut fish = std::fs::File::create("run.fish").unwrap();
        // gen_completion(Shell::Zsh, app, &mut zsh);
        // gen_completion(Shell::Bash, app, &mut bash);
        // gen_completion(Shell::Fish, app, &mut fish);
    }
}
