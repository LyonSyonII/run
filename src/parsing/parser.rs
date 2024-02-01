use crate::command::Command;
use crate::error::Error;
use crate::lang::Language;

use crate::runfile::Runfile;
use crate::fmt::Str;
pub use runfile::runfile;

use crate::HashMap;

enum Element<'i> {
    Command(&'i str, Command<'i>),
    Subcommand(&'i str, Runfile<'i>),
    Include(&'i str, Runfile<'i>),
    Constant(&'i str, Str<'i>),
    Error(Error),
    Errors(Vec<Error>),
}

peg::parser! {
    grammar runfile<'i>() for str {
        rule pos() -> usize = position!()

        rule _ = [' ' | '\t' | '\n' | '\r']+
        rule __ = quiet!{ ([' ' | '\t' | '\n' | '\r'] / comment())* }
        pub rule doc() -> String = c:(("///" c:$([^'\n']*){ c.trim() }) ** "\n") { c.join("\n") }
        pub rule comment() = (!"///" "//" [^'\n']*) ++ "\n" / "/*" (!"*/" [_])* "*/"

        pub rule language() -> Result<Language, Error> = start:pos() i:$([^' '|'\n'|'\t'|'\r'|'('|')'|'{'|'}'|'['|']']+) end:pos() __ ("fn"/"cmd")? {
            i.parse().map_err(|e| Error::PParseLang(e, start, end))
        } / ("fn"/"cmd") {
            Ok(Language::Shell)
        } / start:pos() end:pos() {
            Error::PExpectedLangOrCmd(start, end).err()
        }
        pub rule name() -> Result<&'input str, Error> = __ name:ident() __ {
            Ok(name)
        } / start:pos() end:pos() {
            Error::PExpectedCmdName(start, end).err()
        }
        pub rule ident() -> &'input str = $(['a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '-']+)
        pub rule arguments() -> Result<Vec<&'input str>, Error> = start:pos() s:"("? v:(ident() ** " ") " "? e:")"? end:pos() {
            match (s.is_none(), e.is_none()) {
                (true, false) => Error::PExpectedOpenParen(start, end).err(),
                (false, true) => Error::PExpectedCloseParen(start, end).err(),
                (true, true) => Error::PExpectedArgs(start, end).err(),
                (false, false) => Ok(v)
            }
        }
        pub rule body_start() -> usize = s:$['{']+ { s.len() } /*
        } / start:pos() end:pos() {
            Error::PExpectedBodyStart(start, end).err()
        } */
        pub rule body_end(count: usize) = ['}']*<{count}>
        pub rule body(count: usize) -> &'input str = $((!(['{'|'}']*<{count}>)[_] / "{"*<1, {(count-1).max(1)}> body((count-1).max(1)) "}"*<1, {(count-1).max(1)}>)*)               // TODO: Remove this atrocity
        pub rule command() -> Element<'input> = __ doc:doc() __ lang:language() __ name:name() __ args:arguments() __ count:body_start() script:body(count) body_end(count) __ {
            let mut errors = Vec::new();
            fn unwrap<T>(result: Result<T, Error>, default: T, errors: &mut Vec<Error>) -> T {
                match result {
                    Ok(v) => v,
                    Err(e) => {
                        errors.push(e);
                        default
                    }
                }
            }

            let lang = unwrap(lang, Language::Shell, &mut errors);
            let name = unwrap(name, "", &mut errors);
            let args = unwrap(args, Vec::new(), &mut errors);
            // unwrap(count, 0, &mut errors);

            if errors.is_empty() {
                let command = Command::new(name, doc, lang, args, script);
                Element::Command(name, command)
            } else {
                Element::Errors(errors)
            }
        }
        pub rule subcommand(dir: &std::path::Path) -> Element<'input> = __ doc:doc() __ "sub" __ name:ident() __ "{" sub:runfile(dir) "}" __ {
            match sub {
                Ok(sub) => Element::Subcommand(name, sub.with_doc(doc)),
                Err(e) => Element::Errors(e)
            }
        }

        pub rule include(dir: &std::path::Path) -> Element<'input> = __ "in" __ start:pos() name:($([^'\n']+)) end:pos() __ {
            // TODO: Remove leak (should not impact a lot, the string will need to be alive the whole program anyway)
            let path = {
                if name.starts_with('/') {
                    std::path::PathBuf::from(name)
                } else {
                    dir.join(name)
                }
            };
            let file = match std::fs::read_to_string(&path) {
                Ok(file) => file.leak(),
                Err(e) => return Error::PIncludeRead(e.to_string(), name.to_string(), start, end).into()
            };
            let include = match runfile::runfile(file, path.parent().unwrap_or(dir)) {
                Ok(include) => include,
                Err(e) => return Error::PIncludeParse(e.to_string(), name.to_string(), start, end).into(),
            };
            match include {
                Ok(include) => Element::Include(name, include),
                Err(e) => Element::Errors(e)
            }
        }
        rule dqc() = "\\\"" / [^'"']
        rule sqc() = "\\\'" / [^'\'']
        pub rule value() -> &'input str = ['"'] v:$(dqc()*) ['"'] { v } / "'" v:$(sqc()*) "'" { v } / v:$([^'\n']*)
        pub rule math() -> Result<Str<'input>, Error> = start:pos() "$(" e:$((!(")"[' ']*"\n") [_])*) ")" end:pos() {
            match arithmetic::calculate(e.trim()) {
                Ok(v) => Ok(v.to_string().into()),
                Err(e) => Error::PMathExpression(start, end).err()
            }
        }
        pub rule var() -> Element<'input> = __ "const" _ name:ident() __ "=" __ v:(m:math() { m }/v:value() { Ok(Str::from(v)) }) __ {
            match v {
                Ok(v) => Element::Constant(name, v),
                Err(e) => Element::Error(e)
            }
        }
        pub rule runfile(dir: &std::path::Path) -> Result<Runfile<'input>, Vec<Error>> = __ elements:(var()/include(dir)/subcommand(dir)/command())* __ {
            let mut commands = HashMap::with_hasher(xxhash_rust::xxh3::Xxh3Builder::new());
            let mut subcommands = HashMap::with_hasher(xxhash_rust::xxh3::Xxh3Builder::new());
            let mut includes = HashMap::with_hasher(xxhash_rust::xxh3::Xxh3Builder::new());
            let mut vars = Vec::new();
            let mut errors = Vec::new();
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
                    Element::Constant(name, value) => {
                        vars.push((name, value));
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
                    vars
                }
            )
        }
    }
}

peg::parser!( grammar arithmetic() for str {
    rule _ = [' ' | '\t' | '\n' | '\r']*
    pub(crate) rule calculate() -> f64 = precedence!{
        x:(@) _ "+" _ y:@ { x + y }
        x:(@) _ "-" _ y:@ { x - y }
              "-" _ v:@ { - v }
        --
        x:(@) _ "*" _ y:@ { x * y }
        x:(@) _ "/" _  y:@ { x / y }
        --
        x:@ _ "^" _ y:(@) { x.powf(y) }
        v:@ _ "!"       { (1..v as i64+1).product::<i64>() as f64 }
        --
        "(" _ v:calculate() _ ")" { v }
        n:number() { n }
    }

    rule number() -> f64
        = n:$(['0'..='9']+"."?['0'..='9']* / ['0'..='9']*"."?['0'..='9']+) { n.parse().unwrap() }
});

impl From<Error> for Element<'_> {
    fn from(e: Error) -> Self {
        Element::Error(e)
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
