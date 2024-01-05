use crate::{
    runner::{Command, Language},
    Goodbye, Runfile,
};

peg::parser! {
    pub grammar runfile() for str {
        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = [' ' | '\t' | '\n' | '\r']*

        rule language() -> Language = !"fn" i:ident() { i.parse().goodbye(format!("unknown language '{i}'")) }
        rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+)
        rule arguments() -> Vec<&'input str> = "(" v:(ident() ** ",") ","? ")" { v }
        rule body() -> &'input str = $(([^ '{' | '}'] / "{" body() "}")*)
        rule command() -> (&'input str, Command<'input>) = __ lang:(language() / { Language::Bash }) __ "fn" __ name:ident() __ args:arguments() __ "{" script:body() "}" __ {
           (name, Command::new(lang, args, script))
        }
        pub rule parse() -> Runfile<'input> = __ c:(command()+) __ {
            Runfile {
                commands: c.into_iter().collect()
            }
        }
    }
}
