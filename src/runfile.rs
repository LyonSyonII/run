use std::collections::HashMap;
use std::format as f;

use crate::command::Command;
use crate::utils::{Goodbye as _, OptionExt};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Runfile<'i> {
    pub(crate) commands: HashMap<&'i str, Command<'i>>,
    pub(crate) subcommands: HashMap<&'i str, Runfile<'i>>,
    pub(crate) includes: HashMap<&'i str, Runfile<'i>>,
}

impl<'i> Runfile<'i> {
    fn print_available_commands(&self) {
        println!("Available commands:");
        let mut commands = self.commands.values().collect::<Vec<_>>();
        commands.sort_by(|a, b| {
            if a.name == "default" {
                std::cmp::Ordering::Less
            } else {
                a.name.cmp(b.name)
            }
        });
        let max = commands
            .iter()
            .map(|c| c.name.len())
            .max()
            .unwrap_or_default();
        for cmd in commands {
            let doc = cmd.doc();
            let mut lines = doc.lines();
            println!("    {:max$}   {}", cmd.name, lines.next().unwrap(),);
            for l in lines {
                println!("    {:max$}   {}", " ", l);
            }
        }
    }
    
    #[momo::momo]
    pub fn run(&self, args: impl AsRef<[String]>) {
        let first = args.first();

        if first.is_some_and_oneof(["-h", "--help"]) {
            println!("Runs a runfile in the current directory");
            println!("Possible runfile names: [runfile, run, Runfile, Run]\n");
            println!("Usage: run [COMMAND] [ARGS...]\n");
            println!("Options:");
            println!("  -h, --help\t\tPrints help information");
            println!("  -c, --commands\tPrints available commands in the runfile");
            return;
        } else if first.is_some_and_oneof(["-c", "--commands"]) {
            self.print_available_commands();
            return;
        }

        if let Some(cmd) = first.and_then(|c| self.commands.get(c.as_str())) {
            cmd.run(args.get(1..).unwrap_or_default())
        } else {
            let cmd = self.commands.get("default").bye_and(|| {
                println!("Could not find default subcommand");
                self.print_available_commands();
            });
            cmd.run(args)
        }
        .expect("Command execution failed");
    }
}