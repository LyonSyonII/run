pub use std::format as fmt;
use std::io::Write;

use colored::{Color, Colorize as _};

use crate::{
    lang::Language,
    strlist::{StrList, StrListSlice},
};

#[derive(Eq, Clone)]
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
    
    pub fn args(&self) -> &[&'i str] {
        &self.args
    }

    // Clippy does not detect the usage in the 'format!' macro
    #[allow(unused_variables)]
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

    pub fn doc_raw(&self) -> &str {
        &self.doc
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

    pub fn script_with_indent_fix(&self) -> String {
        // Remove extra indentation from script
        let script = self.script.to_string();
        let mut script = script.lines().filter(|l| !l.trim().is_empty()).peekable();
        let indent = script
            .peek()
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);
        script.map(|l| &l[indent..]).collect::<Vec<_>>().join("\n")
    }

    pub fn run(
        &self,
        parents: StrListSlice,
        args: impl AsRef<[String]>,
        runfile_docs: String,
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
        let mut script = self.script_with_indent_fix();

        // Replace arguments
        for (name, arg) in self.args.iter().zip(args.as_ref()) {
            let name = fmt!("${name}");
            script = script.replace(&name, arg);
        }

        script = script.replace("$doc", &runfile_docs);
        script = script.replace("$cmddoc", &self.doc(parents).to_string());
        script = script.replace("$usage", &self.usage(parents, Color::White, 0));

        if let Err(e) = self.lang.execute(&script) {
            eprintln!(
                "{} {} {}{}\n",
                "Error running".bright_red().bold(),
                parents.color(Color::BrightRed).bold(),
                name.bright_red().bold(),
                ":".bright_red().bold()
            );
            eprintln!("{e}");
        }

        Ok(())
    }
}

impl PartialEq for Command<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.doc == other.doc
            && self.lang == other.lang
            && self.args == other.args
            && self.script_with_indent_fix() == other.script_with_indent_fix()
    }
}

impl PartialOrd for Command<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Command<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.name == "default" {
            return std::cmp::Ordering::Less;
        }
        self.name.cmp(other.name)
    }
}

impl std::fmt::Debug for Command<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("doc", &self.doc)
            .field("lang", &self.lang)
            .field("args", &self.args)
            .field("script", &self.script_with_indent_fix())
            .finish()
    }
}
