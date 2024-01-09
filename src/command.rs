pub use std::format as fmt;
use std::{io::Write, str::FromStr};

use crate::{strlist::StrList, utils::Goodbye};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Command<'i> {
    pub name: &'i str,
    doc: String,
    lang: Language,
    args: Vec<&'i str>,
    script: &'i str,
}

impl<'i> Command<'i> {
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

    pub fn usage(&self, parents: &StrList) -> String {
        let name = self.name;
        let args = self
            .args
            .iter()
            .map(|a| fmt!("<{}>", a.to_uppercase()))
            .reduce(|acc, s| fmt!("{acc} {s}"))
            .unwrap_or_default();
        if name == "default" {
            return fmt!("Usage: {parents} {args}");
        }
        fmt!("Usage: {parents} {name} {args}")
    }
    
    pub fn doc(&self, parents: &StrList) -> std::borrow::Cow<'_, str> {
        let mut lines = self.doc.lines().collect::<Vec<_>>();
        let last = lines.last().cloned().unwrap_or_default();
        if !last.starts_with("Usage:") {
            let usage = self.usage(parents);
            lines.push(&usage);
            lines.join("\n").into()
        } else {
            std::borrow::Cow::Borrowed(&self.doc)
        }
    }

    #[momo::momo]
    pub fn run(&self, parents: &StrList, args: impl AsRef<[String]>) -> std::io::Result<()> {
        let name = self.name;
        if args.iter().any(|a| a == "--help" || a == "-h") {
            println!("{}", self.doc);
            return Ok(());
        }

        if args.len() < self.args.len() {
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
        script = script.replace("$doc", &self.doc(parents));
        script = script.replace("$usage", &self.usage(parents));

        let cmd = match self.lang {
            Language::Bash => "bash",
            Language::Rust => "rustc",
            Language::Python => "python",
            Language::Javascript => "node",
        };

        let mut cmd = std::process::Command::new(cmd)
            .stdin(std::process::Stdio::piped())
            // .args(args.get(self.args.len()..).unwrap_or_default())
            .spawn()?;
        cmd.stdin
            .as_mut()
            .bye("ERROR: Could not take stdin")
            .write_all(script.as_bytes())?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    Bash,
    Rust,
    Python,
    Javascript,
}

impl FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cmd" | "fn" | "sh" | "bash" | "shell" => Ok(Self::Bash),
            "rs" | "rust" => Ok(Self::Rust),
            "py" | "python" => Ok(Self::Python),
            "js" | "javascript" => Ok(Self::Javascript),
            _ => Err(fmt!("Unknown language '{s}'; expected one of [fn, sh, bash, shell, rs, rust, py, python, js, javascript]")),
        }
    }
}
