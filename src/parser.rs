use std::collections::HashMap;
use crate::Runfile;

peg::parser! {
    pub grammar runfile() for str {
        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = [' ' | '\t' | '\n' | '\r']*
        
        rule shebang() -> &'input str = "#!" s:$([^'\n']+) { s }
        rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_']+)
        rule arguments() -> Vec<&'input str> = "(" v:(ident() ** ",") ","? ")" { v }
        rule body() -> &'input str = $(([^ '{' | '}'] / "{" body() "}")+)
        rule command() -> (&'input str, (Vec<&'input str>, &'input str)) = "function" _ name:ident() args:arguments() __ "{" script:body() "}" {
           (name, (args, script))
        }
        pub rule parse() -> Runfile<'input> = shebang:shebang()? [' ' | '\t']* "\n"+ __ c:(command()+) {
            Runfile { 
                shebang, 
                commands: c.into_iter().collect() 
            } 
        }
    }
}