pub use std::format as fmt;
use std::{hash::Hash, io::Write};

// use colored::{Color, Colorize as _};
use yansi::{Color, Paint as _};

use crate::{
    fmt::{
        strlist::{FmtList, FmtListSlice, StrList, StrListSlice},
        Str,
    },
    lang::{Lang, Language},
};

#[derive(Eq, Clone)]
pub struct Command<'i> {
    name: &'i str,
    doc: String,
    lang: Lang,
    args: Vec<&'i str>,
    script: &'i str,
}

impl<'i> Command<'i> {
    // TODO: Add way to specify working directory
    pub fn new(
        name: &'i str,
        doc: String,
        lang: Lang,
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

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn args(&self) -> &[&'i str] {
        &self.args
    }

    // Clippy does not detect the usage in the 'format!' macro
    #[allow(unused_variables)]
    pub fn usage(&self, parents: &StrListSlice, color: Color, newlines: usize) -> String {
        let usage = "Usage:".paint(color).bold();
        let parents = parents.bright_cyan().bold();
        let name = self.name.bright_cyan().bold();
        let args = self.args.iter().fold(String::new(), |acc, a| {
            acc + "<" + &a.to_uppercase() + ">" + " "
        });
        let args = args.cyan();
        if name.value == "default" {
            return format!("{usage} {parents} {args}{}", "\n".repeat(newlines));
        }
        format!("{}{usage} {parents} {name} {args}", "\n".repeat(newlines))
    }

    pub fn doc_raw(&self) -> &str {
        &self.doc
    }

    pub fn doc(&'i self, parents: &StrListSlice) -> FmtList<&'static str, String> {
        let usage = self.usage(parents, Color::White, 0);
        FmtList::from(("\n", std::iter::once(usage))).extend(self.doc.lines())
    }

    pub fn print_help(
        &self,
        parents: StrListSlice,
        indent: usize,
        to: &mut impl Write,
    ) -> std::io::Result<()> {
        let lines = FmtList::<&'static str, &str>::from(("\n", self.doc.lines()));
        let usage = self.usage(&parents, Color::BrightGreen, !lines.is_empty() as usize);

        for l in lines.append(usage.as_str()) {
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

    fn print_expected_args<'a, D>(&'a self, parents: impl std::fmt::Display, got: impl IntoIterator<Item = &'a D> + Clone) -> !
    where
        D: std::fmt::Display + ?Sized + 'a,
    {
        let name = self.name;
        
        let expected = crate::fmt::strlist::FmtList::<&'static str, String>::from((
            ", ",
            self.args.iter().map(|a| format!("<{}>", a.to_uppercase())),
        ));
        let got = crate::fmt::strlist::FmtIter::new(&", ", got);
        eprintln!(
            "{}{parents} {name}: Expected arguments [{expected}], got [{got}]{}",
            "".bright_red().bold().linger(),
            "".resetting()
        );
        eprintln!(
            "See '{}{parents} {name} --help{}' for more information",
            "".bright_cyan().bold(),
            "".resetting()
        );
        std::process::exit(1)
    }

    pub fn run<'a>(
        &'a self,
        parents: StrListSlice,
        args: impl AsRef<[String]>,
        commands: &crate::HashMap<&'i str, Command<'i>>,
        vars: impl AsRef<[&'a (&'i str, Str<'i>)]>,
        runfile_docs: String,
    ) -> std::io::Result<()> {
        let args = args.as_ref();
        let name = self.name;
        if args.iter().any(|a| a == "--help" || a == "-h") {
            self.print_help(parents, 0, &mut std::io::stdout())?;
            return Ok(());
        }

        if args.len() < self.args.len() {
            self.print_expected_args(&parents, args)
        }

        // Remove indentation from script
        let script = replace_all(
            self.name,
            self.lang,
            self.script_with_indent_fix(),
            (&self.args, &args[..self.args.len()]),
            commands,
            vars,
            runfile_docs,
            self.doc(&parents).to_string(),
            self.usage(&parents, Color::White, 0),
        );
        let args = args.get(self.args.len()..).unwrap_or(&[]);
        // Run the script
        if let Err(e) = self.lang.execute(&script, args) {
            eprintln!(
                "{}{} {}{}\n",
                "Error running '".bright_red().bold(),
                parents.magenta().bold(),
                name.magenta().bold(),
                "':".bright_red().bold()
            );
            eprintln!("{e}");
        }

        Ok(())
    }
}

fn replace_all<'a, 'i: 'a>(
    command_name: &'i str,
    lang: Lang,
    mut script: String,
    args: (&[&str], &[String]),
    commands: &crate::HashMap<&'i str, Command<'i>>,
    vars: impl AsRef<[&'a (&'i str, Str<'i>)]>,
    runfile_docs: String,
    doc: String,
    usage: String,
) -> String {
    // Replace arguments
    type Bytes<'i> = beef::lean::Cow<'i, [u8]>;
    let vars = vars.as_ref();

    // TODO: Fix command calls from subcommands
    for (name, command) in commands {
        let re = regex::Regex::new(&format!("\\${}\\(((?:[^\\s]*?\\s*)*?)\\)", name)).unwrap();

        while let Some(c) = re.captures(&script) {
            let r#match = c.get(0).unwrap();
            let (_, [args]) = c.extract();
            
            // TODO: Remove check? To avoid errors on comments and code that will not be executed
            if args.len() < command.args.len() {
                command.print_expected_args(command_name, args.split_ascii_whitespace());
            }
            let replace = lang.command_call(name, args.split_ascii_whitespace());
            script.replace_range(r#match.start()..=r#match.end(), &replace);
        }
    }

    let vars_names = vars.iter().map(|(n, _)| Bytes::owned(fmt!("${n}").into()));
    let vars_values = vars.iter().map(|(_, v)| {
        let patterns = ["\\n", "\\r", "\\t", "\\0", "\\\"", "\\'", "\\\\", "\\$"];
        let replace_with = ["\n", "\r", "\t", "\0", "\"", "'", "\\", "$"];
        let ac = aho_corasick::AhoCorasick::new(patterns).unwrap();
        ac.replace_all(v, &replace_with).into()
    });

    let args_names = args
        .0
        .iter()
        .map(|n| Bytes::owned(fmt!("${n}").into_bytes()));
    let args_values = args.1.iter().map(|v| Str::borrowed(v));

    let patterns = args_names.chain(vars_names).chain([
        Bytes::borrowed(b"$doc"),
        Bytes::borrowed(b"$cmddoc"),
        Bytes::borrowed(b"$usage"),
    ]);

    let replace_with = args_values.chain(vars_values).chain([
        Str::owned(runfile_docs),
        Str::owned(doc),
        Str::owned(usage),
    ]);

    let ac = aho_corasick::AhoCorasick::builder()
        .match_kind(aho_corasick::MatchKind::LeftmostLongest)
        .build(patterns)
        .unwrap();
    ac.replace_all(&script, &replace_with.collect::<Vec<_>>())
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
