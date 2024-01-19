pub use runfile::runfile;
use std::format as fmt;
use crate::lang::Language;
use crate::command::Command;
use crate::runfile::Runfile;
use crate::utils::Goodbye;

peg::parser! {
    grammar runfile() for str {
        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = [' ' | '\t' | '\n' | '\r']*
        pub rule doc() -> String = c:(("///" c:$([^'\n']*){ c.trim() }) ** "\n") { c.join("\n") }
        pub rule comment() = (!"///" "//" [^'\n']*) ** "\n" / "/*" (!"*/" [_])* "*/"
        
        pub rule language() -> Language = !("fn"/"cmd") i:ident() { i.parse().byefmt(|| fmt!("Unknown language '{i}'")) }
        pub rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+)
        pub rule arguments() -> Vec<&'input str> = "(" v:(ident() ** ",") ","? ")" { v }
        pub rule body_start() -> usize = s:$['{']+ { s.len() }
        pub rule body(count: usize) -> &'input str = $((!(['}']*<{count}>)[_])*)
        pub rule command() -> (&'input str, Command<'input>) = __ doc:doc() __ lang:(language() / { Language::Bash }) __ ("fn"/"cmd") __ name:ident() __ args:arguments() __ count:body_start() script:body(count) ['}']*<{count}> __ {
           (name, Command::new(name, doc, lang, args, script))
        }
        pub rule runfile() -> Runfile<'input> = __ comment() c:command()* __ {
            Runfile {
                doc: "".into(),
                commands: c.into_iter().collect(),
                subcommands: std::collections::HashMap::new(),
                includes: std::collections::HashMap::new(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::runfile as p;

    #[test]
    fn doc() {
        assert_eq!(p::doc("#"), Ok("".into()));
        assert_eq!(p::doc("# Comment"), Ok("Comment".into()));
        assert_eq!(
            p::doc("# Pretty long comment :)"),
            Ok("Pretty long comment :)".into())
        );
        assert_eq!(
            p::doc("# First line\n# Second line"),
            Ok("First line\nSecond line".into())
        );
        assert_eq!(
            p::doc("# Example hello world in bash\n# Usage: sh <NAME>"),
            Ok("Example hello world in bash\nUsage: sh <NAME>".into())
        );
    }
}
