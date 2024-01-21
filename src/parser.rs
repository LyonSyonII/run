use crate::command::Command;
use crate::lang::Language;
use crate::runfile::Runfile;
use crate::utils::Goodbye;
use crate::error::Error;
pub use runfile::runfile;
use std::collections::HashMap;
use std::format as fmt;

enum Element<'i> {
    Command(&'i str, Command<'i>),
    Subcommand(&'i str, Runfile<'i>),
    Include(&'i str, Runfile<'i>),
    Error(Error),
    Errors(Vec<Error>)
}

peg::parser! {
    grammar runfile() for str {
        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = quiet!{ ([' ' | '\t' | '\n' | '\r'] / comment())* }
        pub rule doc() -> String = c:(("///" c:$([^'\n']*){ c.trim() }) ** "\n") { c.join("\n") }
        pub rule comment() = (!"///" "//" [^'\n']*) ++ "\n" / "/*" (!"*/" [_])* "*/"
        
        pub rule language() -> Result<Language, Error> = start:position!() i:ident() end:position!() __ ("fn"/"cmd") {
            i.parse().map_err(|e| Error::new(e, start, end))
        } / ("fn"/"cmd") {
            Ok(Language::Shell)
        } / start:position!() end:position!() {
            Error::err("Expected language or fn/cmd", start, end)
        }
        pub rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-']+)
        pub rule arguments() -> Vec<&'input str> = "(" v:(ident() ** " ") " "? ")" { v }
        pub rule body_start() -> usize = s:$['{']+ { s.len() }
        pub rule body(count: usize) -> &'input str = $((!(['{'|'}']*<{count}>)[_] / "{"*<1, {(count-1).max(1)}> body((count-1).max(1)) "}"*<1, {(count-1).max(1)}>)*)
        pub rule command() -> Element<'input> = __ doc:doc() __ lang:language() __ name:ident() __ args:arguments() __ count:body_start() script:body(count) ['}']*<{count}> __ {
           let mut errors = Vec::new();
           let lang = match lang {
               Ok(lang) => lang,
               Err(e) => {
                   errors.push(e);
                   Language::Shell
               }
           };
           
           if errors.is_empty() {
               let command = Command::new(name, doc, lang, args, script);
               Element::Command(name, command)
           } else {
               Element::Errors(errors)
           }
        }
        pub rule subcommand() -> Element<'input> = __ doc:doc() __ "sub" __ name:ident() __ "{" sub:runfile() "}" __ {
            match sub {
                Ok(sub) => Element::Subcommand(name, sub.with_doc(doc)),
                Err(e) => Element::Errors(e)
            }
        }
        pub rule include() -> Element<'input> = __ "in" __ name:($([^'\n']+)) __ {
            // TODO: Remove leak
            let file = std::fs::read_to_string(name).byefmt(|| fmt!("Could not read file '{name}'")).leak();
            let include = runfile::runfile(file).byefmt(|| fmt!("Could not parse file '{name}'"));
            match include {
                Ok(include) => Element::Include(name, include),
                Err(e) => Element::Errors(e)
            }
        }
        pub rule runfile() -> Result<Runfile<'input>, Vec<Error>> = __ elements:(include()/subcommand()/command())* __ {
            let mut errors = Vec::new();
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
                    Element::Error(e) => {
                        errors.push(e);
                    }
                    Element::Errors(e) => {
                        errors.extend(e)
                    }
                }
            }
            if !errors.is_empty() {
                return Err(errors);
            }
            Ok(
                Runfile {
                    doc: String::new(),
                    commands,
                    subcommands,
                    includes,
                }
            )
        }
    }
}

#[cfg(test)]
mod test {
    use super::runfile as p;

    #[test]
    fn doc() {
        assert_eq!(p::doc("///"), Ok("".into()));
        assert_eq!(p::doc("/// Comment"), Ok("Comment".into()));
        assert_eq!(
            p::doc("/// Pretty long comment :)"),
            Ok("Pretty long comment :)".into())
        );
        assert_eq!(
            p::doc("/// First line\n/// Second line"),
            Ok("First line\nSecond line".into())
        );
        assert_eq!(
            p::doc("/// Example hello world in bash\n/// Usage: sh <NAME>"),
            Ok("Example hello world in bash\nUsage: sh <NAME>".into())
        );
    }
}
