use std::{io::Write, str::FromStr};

use crate::Goodbye;

pub struct Command<'i> {
    lang: Language,
    args: Vec<&'i str>,
    script: &'i str,
}

impl<'i> Command<'i> {
    pub fn new(lang: Language, args: Vec<&'i str>, script: &'i str) -> Self {
        Self { lang, args, script }
    }

    pub fn run(&self, args: impl AsRef<[String]>) -> std::io::Result<()> {
        let args = args.as_ref();
        if args.len() != self.args.len() {
            println!("run: Expected arguments {:?}, got {:?}", self.args, args);
            std::process::exit(1);
        }

        let mut script = self.script.to_string();
        for (name, arg) in self.args.iter().zip(args.as_ref()) {
            let name = format!("${name}");
            script = script.replace(&name, arg);
        }

        let mut script = script.lines().filter(|l| !l.trim().is_empty()).peekable();
        let indent = script
            .peek()
            .map(|l| l.len() - l.trim_start().len())
            .unwrap_or(0);
        let script = script.map(|l| &l[indent..]).collect::<Vec<_>>().join("\n");

        let cmd = match self.lang {
            Language::Bash => "bash",
            Language::Rust => "rustc",
            Language::Python => "python",
            Language::Javascript => "node",
        };

        let mut cmd = std::process::Command::new(cmd)
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        cmd.stdin
            .as_mut()
            .goodbye("ERROR: Could not take stdin")
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
