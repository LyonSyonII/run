use std::collections::HashMap;
use std::format as f;

use colored::{Color, Colorize, Styles};

use crate::command::Command;
use crate::strlist::StrList;
use crate::utils::OptionExt;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Runfile<'i> {
    pub(crate) commands: HashMap<&'i str, Command<'i>>,
    pub(crate) subcommands: HashMap<&'i str, Runfile<'i>>,
    pub(crate) includes: HashMap<&'i str, Runfile<'i>>,
    pub(crate) doc: String,
}

impl<'i> Runfile<'i> {
    fn calculate_indent(&self) -> usize {
        self.commands
            .keys()
            .map(|name| name.len())
            .chain(self.subcommands.keys().map(|name| name.len()))
            .max()
            .unwrap_or_default()
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = doc.into();
        self
    }

    #[momo::momo]
    pub fn doc(&self, name: impl AsRef<str>, parents: &StrList) -> std::borrow::Cow<'_, str> {
        let parents = parents.as_slice().color(Color::Cyan).bold();
        let (name, usage) = if name.is_empty() {
            // Main
            (None, "Usage:".green().bold())
        } else {
            // Subcommand
            (Some(name.to_string().cyan().bold()), "Usage:".bold())
        };

        let mut lines = self.doc.lines().collect::<Vec<_>>();
        let last = lines.last().cloned().unwrap_or_default();
        if !last.starts_with("Usage:") {
            let usage = if let Some(name) = name {
                f!(
                    "{usage} {parents} {name} {}\n",
                    "[COMMAND] [ARGS...]".cyan()
                )
            } else {
                f!("{usage} {parents} {}\n", "[COMMAND] [ARGS...]".cyan())
            };
            lines.push(&usage);
            lines.join("\n").into()
        } else {
            std::borrow::Cow::Borrowed(&self.doc)
        }
    }

    fn print_commands(&self, parents: &StrList, indent: usize) {
        eprintln!("{}", "Commands:".green().bold());
        let mut commands = self.commands.values().collect::<Vec<_>>();
        commands.sort_by(|a, b| {
            if a.name == "default" {
                std::cmp::Ordering::Less
            } else {
                a.name.cmp(b.name)
            }
        });
        for cmd in commands {
            let doc = cmd.doc(parents);
            let mut lines = doc.lines();
            let name = f!("{:indent$}", cmd.name);
            eprintln!("    {}   {}", name.cyan().bold(), lines.next().unwrap(),);
            for l in lines {
                eprintln!("    {:indent$}   {}", " ", l);
            }
        }
    }

    fn print_subcommands(&self, parents: &StrList, indent: usize) {
        if self.subcommands.is_empty() {
            return;
        }

        eprintln!("{}", "Subcommands:".green().bold());
        let mut subcommands = self.subcommands.iter().collect::<Vec<_>>();
        subcommands.sort_unstable_by(|(n1, _), (n2, _)| n1.cmp(n2));
        for (name, sub) in subcommands {
            let doc = sub.doc(name, parents);
            let mut lines = doc.lines();
            let name = f!("{:indent$}", name);
            eprintln!("    {}   {}", name.cyan().bold(), lines.next().unwrap(),);
            for l in lines {
                eprintln!("    {:indent$}   {}", " ", l);
            }
        }
    }

    fn print_help(&self, msg: impl std::fmt::Display, parents: &StrList) {
        let indent = self.calculate_indent();
        eprintln!("{}", msg);
        eprintln!("{}", self.doc("", parents));
        self.print_commands(parents, indent);
        self.print_subcommands(parents, indent);
    }

    #[momo::momo]
    pub fn run<'a>(
        &'a self,
        parents: impl Into<StrList<'a>>,
        args: impl AsRef<[String]>,
    ) -> Result<(), std::borrow::Cow<'static, str>> {
        let first = args.first();

        if first.is_some_and_oneof(["-h", "--help"]) {
            eprintln!("Runs a runfile in the current directory");
            eprintln!("Possible runfile names: [runfile, run, Runfile, Run]\n");
            eprintln!("Usage: run [COMMAND] [ARGS...]\n");
            eprintln!("Options:");
            eprintln!("  -h, --help\t\tPrints help information");
            eprintln!("  -c, --commands\tPrints available commands in the runfile");
            return Ok(());
        } else if first.is_some_and_oneof(["-c", "--commands"]) {
            let indent = self.calculate_indent();
            self.print_commands(&parents, indent);
            self.print_subcommands(&parents, indent);
            return Ok(());
        }

        let Some(first) = first.map(String::as_str) else {
            let Some(cmd) = self.commands.get("default") else {
                self.print_help(
                    "ERROR: No command specified and no default command found".red(),
                    &parents,
                );
                return Ok(());
            };
            return cmd
                .run(&parents, args)
                .map_err(|e| f!("Command execution failed: {}", e).into());
        };

        if let Some(cmd) = self.commands.get(first) {
            cmd.run(&parents, args.get(1..).unwrap_or_default())
                .map_err(|e| f!("Command execution failed: {}", e).into())
        } else if let Some(sub) = self.subcommands.get(first) {
            sub.run(parents.append(first), args.get(1..).unwrap_or_default())
        } else {
            self.print_help(
                f!("ERROR: Could not find command or subcommand: {}", first).red(),
                &parents,
            );
            std::process::exit(1);
        }
    }
}
