use crate::{
    runner::{Command, Language},
    Goodbye, Runfile,
};
pub use runfile::parse;
pub use std::format as fmt;

peg::parser! {
    pub grammar runfile() for str {
        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = [' ' | '\t' | '\n' | '\r']*
        pub rule doc() -> String = c:(("#" c:$([^'\n']*){ c.trim() }) ** "\n") { c.join("\n") }

        pub rule language() -> Language = !"fn" i:ident() { i.parse().byefmt(|| fmt!("Unknown language '{i}'")) }
        pub rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+)
        pub rule arguments() -> Vec<&'input str> = "(" v:(ident() ** ",") ","? ")" { v }
        pub rule body() -> &'input str = $(([^ '{' | '}'] / "{" body() "}")*)
        pub rule command() -> (&'input str, Command<'input>) = __ doc:doc() __ lang:(language() / { Language::Bash }) __ "fn" __ name:ident() __ args:arguments() __ "{" script:body() "}" __ {
           (name, Command::new(doc, lang, args, script))
        }
        pub rule parse() -> Runfile<'input> = __ c:command()* __ {
            Runfile {
                commands: c.into_iter().collect()
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
