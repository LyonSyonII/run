pub use runfile::runfile;
use std::collections::HashMap;
use std::format as fmt;
use crate::lang::Language;
use crate::command::Command;
use crate::runfile::Runfile;
use crate::utils::Goodbye;

enum Element<'i> {
    Command(&'i str, Command<'i>),
    Subcommand(&'i str, Runfile<'i>),
    Include(&'i str, Runfile<'i>),
}

peg::parser! {
    grammar runfile() for str {
        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = ([' ' | '\t' | '\n' | '\r'] / comment())*
        pub rule doc() -> String = c:(("///" c:$([^'\n']*){ c.trim() }) ** "\n") { c.join("\n") }
        pub rule comment() = (!"///" "//" [^'\n']*) ++ "\n" / "/*" (!"*/" [_])* "*/"
        
        pub rule language() -> Language = !("fn"/"cmd") i:ident() { ? i.parse().or(Err("[sh, bash, rs, py, js]")) }
        pub rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-']+)
        pub rule arguments() -> Vec<&'input str> = "(" v:(ident() ** " ") " "? ")" { v }
        pub rule body_start() -> usize = s:$['{']+ { s.len() }
        pub rule body(count: usize) -> &'input str = $((!(['}']*<{count}>)[_])*)
        pub rule command() -> Element<'input> = __ doc:doc() __ lang:(language() / { Language::Bash }) __ ("fn"/"cmd") __ name:ident() __ args:arguments() __ count:body_start() script:body(count) ['}']*<{count}> __ {
           Element::Command(name, Command::new(name, doc, lang, args, script))
        }
        pub rule subcommand() -> Element<'input> = __ doc:doc() __ "sub" __ name:ident() __ "{" sub:runfile() "}" __ {
            Element::Subcommand(name, sub.with_doc(doc))
        }
        pub rule include() -> Element<'input> = __ "in" _ name:$([^'\n']+) __ {
            // TODO: Remove leak
            let file = std::fs::read_to_string(name).byefmt(|| fmt!("Could not read file '{name}'")).leak();
            let include = runfile::runfile(file).byefmt(|| fmt!("Could not parse file '{name}'"));
            Element::Include(name, include)
        }
        pub rule runfile() -> Runfile<'input> = __ elements:(include()/subcommand()/command())* __ {
            let mut commands = HashMap::new();
            let mut subcommands = HashMap::new();
            let mut includes = HashMap::new();
            for element in elements {
                match element {
                    Element::Command(name, command) => {
                        commands.insert(name, command);
                    }
                    Element::Subcommand(name, sub) => {
                        subcommands.insert(name, sub);
                    }
                    Element::Include(name, inc) => {
                        includes.insert(name, inc.clone());
                        commands.extend(inc.commands);
                        subcommands.extend(inc.subcommands);
                    }
                }
            }
            Runfile {
                doc: String::new(),
                commands,
                subcommands,
                includes,
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
