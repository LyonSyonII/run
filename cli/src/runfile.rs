use std::format as f;
use std::io::Write as _;

// use colored::{Color, Colorize};
use yansi::{Color, Paint};

use crate::command::Command;
use crate::fmt::{
    strlist::{StrList, StrListSlice},
    Str,
};
use crate::lang::Language as _;
use crate::utils::OptionExt;
use crate::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Runfile<'i> {
    pub(crate) commands: HashMap<&'i str, Command<'i>>,
    pub(crate) subcommands: HashMap<&'i str, Runfile<'i>>,
    pub(crate) includes: HashMap<&'i str, Runfile<'i>>,
    pub(crate) vars: Vec<(&'i str, Str<'i>)>,
    pub(crate) doc: String,
}

impl<'i> Runfile<'i> {
    fn calculate_indent(&self) -> (usize, usize) {
        let first = self
            .commands
            .values()
            .map(|c| c.lang().as_str().len())
            .max()
            .unwrap_or_default();
        let second = self
            .commands
            .values()
            .map(|c| c.name().len())
            .chain(self.subcommands.keys().map(|name| name.len()))
            .max()
            .unwrap_or_default();

        (first + 3, second + 1)
    }

    pub fn with_doc(mut self, doc: impl Into<String>) -> Self {
        self.doc = doc.into();
        self
    }

    pub fn doc(&self, name: impl AsRef<str>, parents: StrListSlice) -> StrList<'_> {
        let name = name.as_ref();
        let parents = parents.bright_cyan().bold();
        let (name, usage) = if name.is_empty() {
            // Main
            (None, "Usage:".bright_green().bold())
        } else {
            // Subcommand
            (Some(name.bright_cyan().bold()), "Usage:".bold())
        };

        let usage = if let Some(name) = name {
            f!("{usage} {parents} {name} {}", "[COMMAND] [ARGS...]".cyan())
        } else {
            f!("{usage} {parents} {}", "[COMMAND] [ARGS...]".cyan())
        };
        let lines = StrList::from(("\n", std::iter::once(usage)));
        lines.extend(self.doc.lines())
    }

    fn print_commands(
        &self,
        parents: StrListSlice,
        indent: (usize, usize),
        to: &mut (impl std::io::Write + ?Sized),
    ) -> Result<(), Str<'_>> {
        let op = |e: std::io::Error| Str::from(e.to_string());

        if self.commands.is_empty() {
            return Ok(());
        }

        writeln!(to, "{}", "Commands:".bright_green().bold()).map_err(op)?;
        let is_nix = crate::nix::is_nix();
        let mut warnings = Vec::new();
        let (lang_indent, name_indent) = indent;
        for cmd in self.commands.values() {
            let doc = cmd.doc(parents);
            let mut lines = doc.into_iter();

            let first = lines.next().unwrap();
            let lang = cmd.lang();
            // if nix is installed all languages are installed
            let color = if lang.installed() || is_nix {
                Color::Cyan
            } else {
                warnings.push(lang);
                Color::BrightYellow
            };
            let name = cmd.name().bright_cyan().bold();
            writeln!(
                to,
                " {lang:<lang_indent$} {name:<name_indent$} {first}",
                lang = format!("<{lang}>").paint(color)
            )
            .map_err(op)?;
            for l in lines {
                writeln!(to, " {:lang_indent$} {:name_indent$} {}", "", "", l).map_err(op)?;
            }
        }

        if !warnings.is_empty() {
            writeln!(to).map_err(op)?;
            writeln!(to, "{}", "Missing Languages:".bright_yellow().bold()).map_err(op)?;
            writeln!(to, "{}", " Some of the languages in this runfile are not installed.\n Check https://github.com/lyonsyonii/runfile#languages for more information.\n\n Missing:".bright_yellow()).map_err(op)?;
            for lang in warnings {
                writeln!(
                    to,
                    " {} {}",
                    "-".bold().bright_yellow(),
                    lang.as_str().bold().bright_yellow()
                )
                .map_err(op)?;
            }
            // TODO: Fix extra newline when no subcommands are present
            writeln!(to).map_err(op)?;
        }

        Ok(())
    }

    fn print_subcommands(
        &self,
        parents: StrListSlice,
        indent: (usize, usize),
        to: &mut (impl std::io::Write + ?Sized),
    ) -> Result<(), Str<'_>> {
        if self.subcommands.is_empty() {
            return Ok(());
        }

        let op = |e: std::io::Error| Str::from(e.to_string());

        writeln!(to, "{}", "Subcommands:".bright_green().bold()).map_err(op)?;
        let subcommands = self.subcommands.iter().collect::<Vec<_>>();
        let indent = indent.0 + indent.1;
        for (name, sub) in subcommands {
            let mut doc = sub.doc(name, parents);
            let name = name.bright_cyan().bold();
            writeln!(to, " {name:<indent$}  {}", doc.pop_front().unwrap()).map_err(op)?;
            for l in doc {
                // writeln!(to, " {:lang_indent$} {:name_indent$} {l}", "", "").map_err(op)?;
                writeln!(to, "  {:indent$} {l}", "").map_err(op)?;
            }
        }

        Ok(())
    }

    fn print_help(
        &self,
        msg: Option<impl std::fmt::Display>,
        parents: StrListSlice,
        to: &mut (impl std::io::Write + ?Sized),
    ) -> Result<(), Str<'_>> {
        let op = |e: std::io::Error| Str::from(e.to_string());
        let msg = msg.as_ref();

        let indent = self.calculate_indent();

        if let Some(msg) = msg {
            writeln!(to, "{msg}").map_err(op)?;
        }
        writeln!(to, "{}", self.doc("", parents)).map_err(op)?;
        if !self.commands.is_empty() || !self.subcommands.is_empty() {
            writeln!(to).map_err(op)?;
        }
        self.print_commands(parents, indent, to)?;
        self.print_subcommands(parents, indent, to)?;

        Ok(())
    }

    pub fn run<'a>(
        &'a self,
        path: &std::path::Path,
        parents: impl Into<StrList<'a>>,
        args: &'a [String],
    ) -> Result<(), Str<'a>> {
        let parents = parents.into();

        std::env::set_current_dir(path).map_err(|e| Str::from(e.to_string()))?;
        
        let first = args.first();
        // Needed for subcommands
        if first.is_some_and_oneof(["-h", "--help"]) {
            self.print_help(None::<&str>, parents.as_slice(), &mut std::io::stdout())?;
            return Ok(());
        }
        if first.is_some_and_oneof(["-c", "--commands"]) {
            let indent = self.calculate_indent();
            let stdout = std::io::stdout();
            let mut stdout = stdout.lock();
            self.print_commands(parents.as_slice(), indent, &mut stdout)?;
            self.print_subcommands(parents.as_slice(), indent, &mut stdout)?;
            stdout.flush().unwrap();
            return Ok(());
        }

        let runfile_docs = || {
            let mut buf = Vec::new();
            self.print_help(None::<&str>, parents.as_slice(), &mut buf)
                .unwrap_or_default();
            String::from_utf8(buf).map_err(|e| e.to_string())
        };

        let default = |args| {
            let Some(cmd) = self.commands.get("default") else {
                self.print_help(
                    Some(
                        "Error: No command specified and no default command found"
                            .bright_red()
                            .bold(),
                    ),
                    parents.as_slice(),
                    &mut std::io::stderr(),
                )?;
                return Ok(());
            };
            return cmd
                .run(parents.as_slice(), args, &self.vars, runfile_docs()?)
                .map_err(|e| f!("Command execution failed: {}", e).into());
        };

        let Some(first) = first.map(String::as_str) else {
            return default(args);
        };
        if first == "--" {
            return default(args.get(1..).unwrap_or_default());
        }

        if let Some(cmd) = self.commands.get(first) {
            cmd.run(
                parents.as_slice(),
                args.get(1..).unwrap_or_default(),
                &self.vars,
                runfile_docs()?,
            )
            .map_err(|e| f!("Command execution failed: {}", e).into())
        } else if let Some(sub) = self.subcommands.get(first) {
            sub.run(
                path,
                parents.append(first),
                args.get(1..).unwrap_or_default(),
            )
        } else if self
            .commands
            .get("default")
            .is_some_and(|d| d.args().is_empty())
        {
            let meant = "If you meant to run the default command with extra arguments, use '--' before the arguments:\nrun --".white().dim().linger();
            self.print_help(
                Some(
                    format_args!(
                        "Error: Could not find command or subcommand '{}'\n{meant} {}",
                        first,
                        StrList::from((" ", args.iter().map(String::as_str))).clear()
                    )
                    .bright_red()
                    .bold(),
                ),
                parents.as_slice(),
                &mut std::io::stderr(),
            )?;
            std::process::exit(1);
        } else {
            default(args)
        }
    }
}
