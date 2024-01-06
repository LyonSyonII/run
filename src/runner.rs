use crate::Goodbye;
pub use std::format as fmt;
use std::{io::Write, str::FromStr};

pub struct Command<'i> {
    doc: String,
    lang: Language,
    args: Vec<&'i str>,
    script: &'i str,
}

impl<'i> Command<'i> {
    pub fn new(doc: String, lang: Language, args: Vec<&'i str>, script: &'i str) -> Self {
        Self {
            doc,
            lang,
            args,
            script,
        }
    }

    pub fn run(&self, name: impl AsRef<str>, args: impl AsRef<[String]>) -> std::io::Result<()> {
        let name = name.as_ref();
        let args = args.as_ref();
        if args.iter().any(|a| matches!(a.as_str(), "--help" | "-h")) {
            if self.doc.is_empty() {
                let args = self.args.iter().map(|a| fmt!("<{}>", a.to_uppercase())).reduce(|acc, s| fmt!("{acc} {s}")).unwrap_or_default();
                println!("Usage: run {name} {args}");
            } else {
                println!("{}", self.doc);
            }
            return Ok(());
        }

        if args.len() < self.args.len() {
            eprintln!("run {name}: Expected arguments {:?}, got {:?}", self.args, args);
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
        script = script.replace("$doc", &self.doc);

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

pub enum Language {
    Bash,
    Rust,
    Python,
    Javascript,
}

impl FromStr for Language {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fn" | "sh" | "bash" | "shell" => Ok(Self::Bash),
            "rs" | "rust" => Ok(Self::Rust),
            "py" | "python" => Ok(Self::Python),
            "js" | "javascript" => Ok(Self::Javascript),
            _ => Err(()),
        }
    }
}
