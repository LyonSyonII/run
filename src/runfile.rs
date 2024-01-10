use std::collections::HashMap;
use std::format as f;
use std::io::Write as _;

use colored::{Color, Colorize};

use crate::command::Command;
use crate::strlist::{Str, StrList, StrListSlice};
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

    pub fn doc(&self, name: impl AsRef<str>, parents: StrListSlice) -> StrList<'_> {
        let name = name.as_ref();
        let parents = parents.color(Color::BrightCyan).bold();
        let (name, usage) = if name.is_empty() {
            // Main
            (None, "Usage:".bright_green().bold())
        } else {
            // Subcommand
            (Some(name.to_string().bright_cyan().bold()), "Usage:".bold())
        };

        let lines: StrList = ("\n", self.doc.lines()).into();
        let last = lines.last().unwrap_or_default();
        if last.starts_with("Usage:") {
            return lines;
        }

        let usage = if let Some(name) = name {
            f!(
                "{usage} {parents} {name} {}\n",
                "[COMMAND] [ARGS...]".cyan()
            )
        } else {
            f!("{usage} {parents} {}\n", "[COMMAND] [ARGS...]".cyan())
        };
        lines.append(usage)
    }

    fn print_commands(
        &self,
        parents: StrListSlice,
        indent: usize,
        to: &mut (impl std::io::Write + ?Sized),
    ) -> Result<(), Str<'_>> {
        let op = |e: std::io::Error| Str::from(e.to_string());
        
        writeln!(to, "{}", "Commands:".bright_green().bold()).map_err(op)?;
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
            let mut lines = doc.into_iter();
            
            let first = lines.next().unwrap();
            writeln!(to, "    {:indent$}   {}", cmd.name.bright_cyan().bold(), first).map_err(op)?;
            for l in lines {
                writeln!(to, "    {:indent$}   {}", "", l).map_err(op)?;
            }
        }

        Ok(())
    }

    fn print_subcommands(
        &self,
        parents: StrListSlice,
        indent: usize,
        to: &mut (impl std::io::Write + ?Sized),
    ) -> Result<(), Str<'_>> {
        if self.subcommands.is_empty() {
            return Ok(());
        }

        let op = |e: std::io::Error| Str::from(e.to_string());

        writeln!(to, "{}", "Subcommands:".bright_green().bold()).map_err(op)?;
        let mut subcommands = self.subcommands.iter().collect::<Vec<_>>();
        subcommands.sort_unstable_by(|(n1, _), (n2, _)| n1.cmp(n2));
        for (name, sub) in subcommands {
            let mut doc = sub.doc(name, parents);
            let name = f!("{:indent$}", name);
            writeln!(
                to,
                "    {}   {}",
                name.bright_cyan().bold(),
                doc.pop_front().unwrap()
            )
            .map_err(op)?;
            let last = doc.pop();
            for l in doc {
                writeln!(to, "    {:indent$}   {}", " ", l).map_err(op)?;
            }
            if let Some(last) = last {
                write!(to, "    {:indent$}   {}", " ", last).map_err(op)?;
            }
        }

        Ok(())
    }

    fn print_help(
        &self,
        msg: impl AsRef<str>,
        parents: StrListSlice,
        to: &mut (impl std::io::Write + ?Sized),
    ) -> Result<(), Str<'_>> {
        let op = |e: std::io::Error| Str::from(e.to_string());
        let msg = msg.as_ref();

        let indent = self.calculate_indent();

        if !msg.is_empty() {
            writeln!(to, "{}", msg).map_err(op)?;
        }
        writeln!(to, "{}", self.doc("", parents)).map_err(op)?;
        self.print_commands(parents, indent, to)?;
        self.print_subcommands(parents, indent, to)?;

        Ok(())
    }

    pub fn run<'a>(
        &'a self,
        parents: impl Into<StrList<'a>>,
        args: &'a [String],
    ) -> Result<(), Str<'a>> {
        let parents = parents.into();

        let first = args.first();
        if first.is_some_and_oneof(["-h", "--help"]) {
            self.print_help("", parents.as_slice(), &mut std::io::stdout())?;
            return Ok(());
        }
        if first.is_some_and_oneof(["-c", "--commands"]) {
            let indent = self.calculate_indent();
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            self.print_commands(parents.as_slice(), indent, &mut stdout)?;
            self.print_subcommands(parents.as_slice(), indent, &mut stdout)?;
            return Ok(());
        }

        let runfile_docs = |to: &mut Vec<u8>| {
            self.print_help("", parents.as_slice(), to).unwrap_or_default()
        };

        let Some(first) = first.map(String::as_str) else {
            let Some(cmd) = self.commands.get("default") else {
                self.print_help(
                    "Error: No command specified and no default command found".bright_red().bold().to_string(),
                    parents.as_slice(),
                    &mut std::io::stderr()
                )?;
                return Ok(());
            };
            return cmd
                .run(parents.as_slice(), args, runfile_docs)
                .map_err(|e| f!("Command execution failed: {}", e).into());
        };

        if let Some(cmd) = self.commands.get(first) {
            cmd.run(parents.as_slice(), args.get(1..).unwrap_or_default(), runfile_docs)
                .map_err(|e| f!("Command execution failed: {}", e).into())
        } else if let Some(sub) = self.subcommands.get(first) {
            sub.run(parents.append(first), args.get(1..).unwrap_or_default())
        } else {
            self.print_help(
                f!("ERROR: Could not find command or subcommand: {}", first).bright_red().to_string(),
                parents.as_slice(),
                &mut std::io::stderr()
            )?;
            std::process::exit(1);
        }
    }
}
