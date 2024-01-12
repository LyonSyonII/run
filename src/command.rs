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
    name: &'i str,
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

    pub fn name(&self) -> &'i str {
        self.name
    }

    pub fn lang(&self) -> Language {
        self.lang
    }

    pub fn usage(&self, parents: StrListSlice, color: Color, newlines: usize) -> String {
        let usage = "Usage:".color(color).bold();
        let parents = parents.color(Color::BrightCyan).bold();
        let name = self.name.bright_cyan().bold();
        let args = self
            .args
            .iter()
            .map(|a| fmt!("<{}>", a.to_uppercase()))
            .reduce(|acc, s| fmt!("{acc} {s}"))
            .unwrap_or_default()
            .cyan();
        if name.contains("default") {
            return fmt!("{usage} {parents} {args}{:\n<newlines$}", "");
        }
        fmt!("{:\n<newlines$}{usage} {parents} {name} {args}", "")
    }

    pub fn doc(&'i self, parents: StrListSlice) -> StrList<'i> {
        let lines = StrList::from(("\n", self.doc.lines()));
        let last = lines.last().unwrap_or_default();
        if !last.starts_with("Usage:") {
            let usage = self.usage(parents, Color::White, 0);
            lines.append(usage)
        } else {
            lines
        }
    }

    pub fn print_help(
        &self,
        parents: StrListSlice,
        indent: usize,
        to: &mut impl Write,
    ) -> std::io::Result<()> {
        let lines = StrList::from(("\n", self.doc.lines()));
        let usage = self.usage(parents, Color::BrightGreen, !lines.is_empty() as usize);

        for l in lines.append(usage) {
            writeln!(to, "{:indent$}{l}", "")?;
        }

        Ok(())
    }

    pub fn run(
        &self,
        parents: StrListSlice,
        args: impl AsRef<[String]>,
        runfile_docs: impl Fn(&mut Vec<u8>),
    ) -> std::io::Result<()> {
        let args = args.as_ref();
        let name = self.name;
        if args.iter().any(|a| a == "--help" || a == "-h") {
            self.print_help(parents, 0, &mut std::io::stdout())?;
            return Ok(());
        }

        if args.len() < self.args.len() {
            let error = format!(
                "{parents} {name}: Expected arguments {:?}, got {:?}",
                self.args, args
            )
            .bright_red()
            .bold();
            eprintln!("{error}");
            let help = format!("{parents} {name} --help").bold().bright_cyan();
            eprintln!("See '{help}' for more information");
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

        let runfile_docs = {
            let mut buf = Vec::new();
            runfile_docs(&mut buf);
            String::from_utf8(buf).unwrap()
        };

        script = script.replace("$doc", &runfile_docs);
        script = script.replace("$cmddoc", &self.doc(parents).to_string());
        script = script.replace("$usage", &self.usage(parents, Color::White, 0));

        if let Err(e) = self.lang.execute(&script) {
            eprintln!(
                "{} {} {}{}\n",
                "Error running".bright_red().bold(),
                parents.color(Color::Red).bold(),
                name.bright_red().bold(),
                ":".bright_red()
            );
            eprintln!("{e}");
        }

        Ok(())
    }
}
