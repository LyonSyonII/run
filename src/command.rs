pub use std::format as fmt;
use std::{io::Write, str::FromStr};

use colored::{Color, Colorize as _};

use crate::{
    lang::Language,
    strlist::{StrList, StrListSlice},
    utils::Goodbye,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Command<'i> {
    pub name: &'i str,
    doc: String,
    lang: Language,
    args: Vec<&'i str>,
    script: &'i str,
}

impl<'i> Command<'i> {
    // TODO: Add way to specify working directory
    pub fn new(
        name: &'i str,
        doc: String,
        lang: Language,
        args: Vec<&'i str>,
        script: &'i str,
    ) -> Self {
        Self {
            name,
            doc,
            lang,
            args,
            script,
        }
    }

    pub fn usage(&self, parents: StrListSlice) -> String {
        let usage = "Usage:".bold();
        let parents = parents.color(Color::Cyan).bold();
        let name = self.name.cyan().bold();
        let args = self
            .args
            .iter()
            .map(|a| fmt!("<{}>", a.to_uppercase()))
            .reduce(|acc, s| fmt!("{acc} {s}"))
            .unwrap_or_default()
            .cyan();
        if name.contains("default") {
            return fmt!("Usage: {parents} {args}");
        }
        fmt!("{usage} {parents} {name} {args}")
    }

    pub fn doc(&'i self, parents: StrListSlice) -> StrList<'i> {
        let lines = StrList::from(("\n", self.doc.lines()));
        let last = lines.last().unwrap_or_default();
        if !last.starts_with("Usage:") {
            let usage = self.usage(parents);
            lines.append(usage)
        } else {
            lines
        }
    }

    pub fn run(&self, parents: StrListSlice, args: impl AsRef<[String]>) -> std::io::Result<()> {
        let args = args.as_ref();
        let name = self.name;
        if args.iter().any(|a| a == "--help" || a == "-h") {
            println!("{}", self.doc);
            return Ok(());
        }

        if args.len() < self.args.len() {
            // TODO: Make output prettier
            eprintln!(
                "run {name}: Expected arguments {:?}, got {:?}",
                self.args, args
            );
            eprintln!("See `run {name} --help` for more information");
            std::process::exit(1);
        }

        // Remove indentation from script
        let script = self.script.to_string();
        let mut script = script.lines().filter(|l| !l.trim().is_empty()).peekable();
        let indent = script
            .peek()
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);
        let mut script = script.map(|l| &l[indent..]).collect::<Vec<_>>().join("\n");

        // Replace arguments
        for (name, arg) in self.args.iter().zip(args.as_ref()) {
            let name = fmt!("${name}");
            script = script.replace(&name, arg);
        }
        script = script.replace("$doc", &self.doc(parents).to_string());
        script = script.replace("$usage", &self.usage(parents));

        if let Err(e) = self.lang.execute(&script) {
            eprintln!(
                "{} {} {}{}",
                "Error running".red(),
                parents.color(Color::Red).bold(),
                name.red().bold(),
                ":".red()
            );
            eprintln!("{e}");
        }

        Ok(())
    }
}
