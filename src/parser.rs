use crate::{command::Command, lang::Language, runfile::Runfile, utils::BoolExt as _};
use chumsky::{prelude::*, text::Char as _};
pub use std::format as fmt;

type Error<'i> = extra::Err<Rich<'i, char>>;
type Parsed<'i, T> = Boxed<'i, 'i, &'i str, T, Error<'i>>;

fn error<'i>(e: Rich<'i, char>, msg: &'static str) -> Rich<'i, char> {
    Rich::custom(*e.span(), msg)
}

fn string<'i>() -> Parsed<'i, &'i str> {
    let escape = just('\\').then_ignore(one_of("\\/\"bfnrt"));

    none_of("\\\"")
        .or(escape)
        .repeated()
        .to_slice()
        .delimited_by(just('"'), just('"'))
        .boxed()
}

fn line_comment<'i>() -> Parsed<'i, ()> {
    text::inline_whitespace() // indentation
        .ignore_then(
            just("//")
                .then(just('/').not())
                .then_ignore(text::inline_whitespace().or_not()),
        )
        .ignore_then(any().and_is(text::newline().not()).repeated())
        .then(text::newline().or(end()))
        .ignored()
        .boxed()
}

fn block_comment<'i>() -> Parsed<'i, ()> {
    text::inline_whitespace() // indentation
        .ignore_then(just("/*"))
        .ignore_then(just("*/").not().repeated())
        .then_ignore(just("*/"))
        .boxed()
}

fn empty_line<'i>() -> Parsed<'i, ()> {
    text::inline_whitespace()
        .then(text::newline())
        .ignored()
        .boxed()
}

#[test]
fn test_empty_line() {
    assert!(empty_line().parse("").into_result().is_err());
    assert!(empty_line().parse(" ").into_result().is_err());
    assert!(empty_line().parse("\n").into_result().is_ok());
    assert!(empty_line().parse(" \n").into_result().is_ok());
    assert!(empty_line().parse("hola").into_result().is_err());
    assert!(empty_line().parse("hola\n").into_result().is_err());
}

fn comment<'i>() -> Parsed<'i, ()> {
    choice((
        empty_line(),
        line_comment(),
        block_comment(),
    )).boxed()
}

fn doc<'i>() -> Parsed<'i, String> {
    text::inline_whitespace() // indentation
        .ignore_then(just("///").then_ignore(text::inline_whitespace()))
        .ignore_then(any().and_is(text::newline().not()).repeated().to_slice())
        .separated_by(text::newline())
        .allow_trailing()
        .collect::<Vec<&'i str>>()
        .map(|v| v.join("\n"))
        .then(just('\n').or_not())
        .validate(|(doc, newline): (String, Option<char>), e, emitter| {
            if newline.is_some() {
                emitter.emit(Rich::custom(
                    e.span(),
                    "empty line found, documentation must be adjacent",
                ));
            }
            doc
        })
        .boxed()
}

fn indentation<'i>() -> Parsed<'i, usize> {
    just(' ').repeated().count().boxed()
}

fn language_fn<'i>() -> Parsed<'i, (usize, Language)> {
    let lang_ident = any()
        .filter(|c: &char| !c.is_whitespace())
        .repeated()
        .to_slice();
    let cmd = choice((text::keyword("fn"), text::keyword("cmd")));
    let language = cmd.clone().to(Language::default()).or(lang_ident
        .try_map(|s: &str, span| s.parse::<Language>().map_err(|e| Rich::custom(span, e)))
        .then_ignore(text::whitespace().at_least(1))
        .then_ignore(cmd.map_err(|e| error(e, "expected 'fn' or 'cmd'"))));

    indentation().then(language).then_ignore(text::whitespace()).boxed()
}

fn fn_ident<'i>() -> Parsed<'i, &'i str> {
    let start = any()
        // Use try_map over filter to get a better error on failure
        .try_map(|c: char, span| {
            c.is_ident_start().and_ok_or(
                c,
                Rich::custom(span, fmt!("an identifier cannot start with {c:?}")),
            )
        });

    let next = any()
        // This error never appears due to `repeated` so can use `filter`
        .filter(|c: &char| c.is_ident_continue() || *c == '-')
        .repeated();

    start.then(next).to_slice().boxed()
}

fn args<'i>() -> Parsed<'i, Vec<&'i str>> {
    fn_ident()
        .separated_by(text::whitespace().at_least(1))
        .allow_trailing()
        .collect()
        .delimited_by(
            just('(').padded().expect("expected '(' before arguments"),
            just(')').expect("expected ')' after arguments"),
        )
        .boxed()
}

/* fn body<'i>(indent: usize) -> Parsed<'i, &'i str> {
    // ([^ '{' '}'] / "{" body() "}")*

    /*     let body = recursive(|body| {
        choice((
            none_of("{}").to_slice(),
            just('{').then(body).then(just('}')).to_slice(),
        ))
        .repeated()
        .to_slice()
    });

    just('{')
        .ignore_then(body)
        .then_with_ctx(then)
        .then_ignore(just('}'))
        // .map(|b: &str| b.trim()) TODO: Check if it really needs to be removed
        .boxed() */
    let end = just(' ').repeated().configure(|cfg, indent: &usize| cfg.exactly(*indent)).then(just('}'));

    just('{')
        .ignore_then(any().and_is(end.not()).repeated().to_slice())
        .then_ignore(end)
        .boxed()
} */

fn signature<'i>() -> Parsed<'i, ((usize, Language), &'i str, Vec<&'i str>)> {
    language_fn()
        .then(fn_ident().padded().labelled("command name"))
        .then(args())
        .map(|((lang, name), args)| (lang, name, args))
        .boxed()
}

fn command<'i>() -> Parsed<'i, (&'i str, Command<'i>)> {
    let inline_body = {
        let end = just('}')
            .ignore_then(text::inline_whitespace())
            .ignore_then(just('\n').ignored().or(end()));

        let any = any()
            .and_is(choice((text::newline(), end)).not())
            .repeated()
            .to_slice();
        just('{')
            .ignore_then(text::inline_whitespace())
            .ignore_then(any)
            .then_ignore(end)
    };
    let body = {
        let end = just(' ')
            .repeated()
            .configure(|cfg, (_, indent, _, _, _)| {
                cfg.exactly(*indent)
            })
            .then(just('}'));

        let line: Boxed<'_, '_, _, &str, _> = any()
            .and_is(text::newline().not())
            .repeated()
            .then(just('\n'))
            .to_slice()
            .boxed();

        let lines = end.not().then(line).repeated().to_slice();

        let multiline = just('{')
            .ignore_then(text::inline_whitespace())
            .ignore_then(just('\n'))
            .ignore_then(lines)
            .then_ignore(end)
            .then_ignore(text::inline_whitespace())
            .then_ignore(just('\n').ignored().or(chumsky::prelude::end()));
        
        choice((multiline, inline_body))
    };
    doc()
        .then(signature())
        .map(|(doc, ((indent, lang), name, args))| (doc, indent, lang, name, args))
        .then_ignore(text::inline_whitespace())
        .then_with_ctx(body.labelled("command body"))
        .map(|((doc, _, lang, name, args), script)| {
            (name, Command::new(name, doc, lang, args, script))
        })
        .boxed()
}

fn subcommand<'i>(runfile: Parsed<'i, Runfile<'i>>) -> Parsed<'i, (&'i str, Runfile<'i>)> {
    doc()
        .then_ignore(text::keyword("sub").padded().expect("expected 'sub'"))
        .then(fn_ident().padded().expect("expected subcommand name"))
        .then_ignore(just('{').expect("expected '{'"))
        .then(runfile.padded())
        .then_ignore(just('}').expect("expected '}'"))
        .map(|((doc, name), runfile)| (name, runfile.with_doc(doc)))
        .boxed()
}

fn include<'i>() -> Parsed<'i, (&'i str, Runfile<'i>)> {
    doc()
        .then_ignore(text::inline_whitespace())
        .then_ignore(text::keyword("in"))
        .then_ignore(text::inline_whitespace())
        .then(any().and_is(text::newline().not()).repeated().to_slice())
        .validate(|(doc, path), e, emitter| {
            let path = path.trim();
            if path.is_empty() {
                emitter.emit(Rich::custom(e.span(), "expected path to include"));
                return ("", Runfile::default());
            }

            if !doc.is_empty() {
                emitter.emit(Rich::custom(e.span(), "includes cannot have documentation"));
            }

            // TODO: Protect against circular includes
            // TODO: Remove leak (although string is alive until the end of the program, so it shouldn't be a problem)
            // TODO: Read relative to the current file instead of the current directory
            let file = match std::fs::read_to_string(path) {
                Ok(s) => s.leak(),
                Err(err) => {
                    emitter.emit(Rich::custom(e.span(), fmt!("{err}")));
                    return (path, Runfile::default());
                }
            };
            let runfile = match runfile().parse(file).into_result() {
                Ok(r) => r,
                Err(errors) => {
                    let errors = errors
                        .into_iter()
                        .map(|e| fmt!("{e:?}"))
                        .fold(fmt!("include {path} has errors:"), |acc, s| {
                            fmt!("{acc}\n{s}")
                        });
                    emitter.emit(Rich::custom(e.span(), errors));
                    Runfile::default()
                }
            };

            (path, runfile)
        })
        .boxed()
}

pub fn runfile<'i>() -> Parsed<'i, Runfile<'i>> {
    enum Results<'i> {
        Command((&'i str, Command<'i>)),
        Subcommand((&'i str, Runfile<'i>)),
        Include((&'i str, Runfile<'i>)),
    }
    
    // Ignore empty lines
    recursive(|runfile| {
            choice((
                include().map(Results::Include),
                subcommand(runfile.boxed()).map(Results::Subcommand),
                command().map(Results::Command),
            ))
            .separated_by(comment().repeated())
            .allow_leading()
            .allow_trailing()
            .collect::<Vec<Results<'i>>>()
            .map(|results| {
                results
                    .into_iter()
                    .fold(Runfile::default(), |mut acc, new| match new {
                        Results::Command((name, cmd)) => {
                            acc.commands.insert(name, cmd);
                            acc
                        }
                        Results::Subcommand((name, sub)) => {
                            acc.subcommands.insert(name, sub);
                            acc
                        }
                        Results::Include((path, include)) => {
                            // TODO: Avoid clones
                            acc.commands.extend(include.commands.clone());
                            acc.subcommands.extend(include.subcommands.clone());
                            acc.includes.insert(path, include);
                            acc
                        }
                    })
            })
    })
    .boxed()
}

trait ParserExt<'i, T>: Parser<'i, &'i str, T, Error<'i>>
where
    Self: Sized,
{
    fn expect(
        self,
        msg: impl AsRef<str>,
    ) -> chumsky::combinator::MapErr<Self, impl Fn(Rich<'i, char>) -> Rich<'i, char>> {
        self.map_err(move |e: Rich<'i, char>| Rich::custom(*e.span(), msg.as_ref()))
    }

    fn expect_with_start(
        self,
        msg: impl AsRef<str>,
        start: usize,
    ) -> chumsky::combinator::MapErr<Self, impl Fn(Rich<'i, char>) -> Rich<'i, char>> {
        self.map_err(move |e: Rich<'i, char>| {
            Rich::custom((start..e.span().end()).into(), msg.as_ref())
        })
    }
}
impl<'i, P, T> ParserExt<'i, T> for P where P: Parser<'i, &'i str, T, Error<'i>> {}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{command::Command, lang::Language, runfile::Runfile};
    use chumsky::Parser as _;

    #[test]
    fn doc() {
        use super::doc;
        assert_eq!(doc().parse("///").unwrap(), "");
        assert_eq!(doc().parse("/// hola").unwrap(), "hola");
        assert_eq!(doc().parse("/// hola\n").unwrap(), "hola");
        assert_eq!(
            doc().parse("/// hola\n/// patata\n").unwrap(),
            "hola\npatata"
        );
        assert!(doc().parse("/// hola\n\n/// patata").into_result().is_err());
        assert!(doc()
            .parse("/// hola\n/// patata\n\n")
            .into_result()
            .is_err());
    }

    #[test]
    fn language() {
        use super::language_fn;
        assert_eq!(language_fn().parse("bash fn").unwrap(), (0, Language::Bash));
        assert_eq!(language_fn().parse("fn").unwrap(), (0, Language::Shell));
        assert!(language_fn().parse("bas fn").into_result().is_err());
        assert!(language_fn().parse("bash").into_result().is_err());
        assert_eq!(
            language_fn().parse("  bash fn").unwrap(),
            (2, Language::Bash)
        );
        assert_eq!(language_fn().parse("  fn").unwrap(), (2, Language::Shell));
    }

    #[test]
    fn args() {
        use super::args;
        assert_eq!(args().parse("()").unwrap(), Vec::<&str>::new());
        assert_eq!(args().parse("  ()").unwrap(), Vec::<&str>::new());
        assert_eq!(args().parse("(a)").unwrap(), vec!["a"]);
        assert_eq!(args().parse("(a b)").unwrap(), vec!["a", "b"]);
        assert_eq!(args().parse("(a b c)").unwrap(), vec!["a", "b", "c"]);
        assert_eq!(args().parse("(a b c d)").unwrap(), vec!["a", "b", "c", "d"]);
        assert_eq!(args().parse("(   )").unwrap(), Vec::<&str>::new());
        assert_eq!(args().parse("( \n  )").unwrap(), Vec::<&str>::new());
        assert_eq!(args().parse("( a   )").unwrap(), vec!["a"]);
        assert_eq!(args().parse("( a\n )").unwrap(), vec!["a"]);
        assert_eq!(args().parse("(a \n )").unwrap(), vec!["a"]);
        assert_eq!(args().parse("(a   b)").unwrap(), vec!["a", "b"]);
        assert_eq!(args().parse("( a b )").unwrap(), vec!["a", "b"]);
        assert_eq!(args().parse("(a\n b)").unwrap(), vec!["a", "b"]);
        assert_eq!(args().parse("(a \n b)").unwrap(), vec!["a", "b"]);
        assert_eq!(args().parse("(a  \nb)").unwrap(), vec!["a", "b"]);
        assert_eq!(
            args().parse("(\na\nb\nc\nd\n)").unwrap(),
            vec!["a", "b", "c", "d"]
        );
    }

    /*     #[test]
    fn body() {
        use super::body;

        let tests = [
            ("{}", ""),
            ("{potato}", "potato"),
            ("{pot{a}to}", "pot{a}to"),
            ("{po{t{a}}to}", "po{t{a}}to"),
        ];

        for test in tests {
            match body().parse(test.0).into_result() {
                Ok(s) => assert_eq!(s, test.1),
                Err(e) => panic!("Error parsing '{}': {:?}", test.0, e),
            }
        }

        let errors = [
            "{",
            "}",
            "{potato",
            "potato}",
            "{pot{a}to",
            "pot{a}to}",
            "{po{t{a}}to",
            "po{t{a}}to}",
            "{}{}{}",
        ];

        for error in errors {
            assert!(
                body().parse(error).has_errors(),
                "Expected error parsing '{}'",
                error
            );
        }
    } */

    #[test]
    fn indentation() {
        assert_eq!(super::indentation().parse("").unwrap(), 0);
        assert_eq!(super::indentation().parse(" ").unwrap(), 1);
        assert_eq!(super::indentation().parse("  ").unwrap(), 2);
        assert_eq!(super::indentation().parse("   ").unwrap(), 3);
        assert_eq!(super::indentation().parse("    ").unwrap(), 4);
    }

    #[test]
    fn signature() {
        use super::signature;
        let expected_signature = ((0, Language::Bash), "greet", vec!["name"]);
        let actual_signature = signature().parse("bash fn greet(name)").unwrap();
        assert_eq!(actual_signature, expected_signature);

        let expected_signature = ((0, Language::Shell), "pata", vec!["name", "age"]);
        let actual_signature = signature().parse("fn pata (name age)").unwrap();
        assert_eq!(actual_signature, expected_signature);

        let expected_signature = ((2, Language::Bash), "greet", vec!["name"]);
        let actual_signature = signature().parse("  bash fn greet(name)").unwrap();
        assert_eq!(actual_signature, expected_signature);

        let expected_signature = ((2, Language::Shell), "pata", vec!["name", "age"]);
        let actual_signature = signature().parse("  fn pata (name age)").unwrap();
        assert_eq!(actual_signature, expected_signature);
    }

    #[test]
    fn command() {
        use super::command;
        let expected_command = Command::new(
            "inline",
            "inline command".into(),
            Language::Shell,
            vec!["name"],
            "echo 'Hello, $name.sh'; ",
        );
        let actual_command = command()
            .parse("/// inline command\nsh fn inline(name) { echo 'Hello, $name.sh'; }")
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("inline", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );

        let expected_command = Command::new(
            "multiline",
            "".into(),
            Language::Shell,
            vec!["name"],
            "echo 'Hello, $name.sh';",
        );
        let actual_command = command()
            .parse(
                "sh fn multiline(name) {\necho 'Hello, $name.sh';\n}"
            )
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("multiline", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );

        let actual_command = command()
        .parse(
            "  sh fn multiline(name) { \n  echo 'Hello, $name.sh';\n  }"
        )
        .into_result();
        assert_eq!(
            actual_command,
            Ok(("multiline", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );
        
        let expected_command = Command::new(
            "multiline",
            "multiline command".into(),
            Language::Shell,
            vec!["name"],
            "echo 'Hello, $name.sh';",
        );
        let actual_command = command()
            .parse(
                
r#"/// multiline command
sh fn multiline(name) { 
  echo 'Hello, $name.sh';
}"#,
            )
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("multiline", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );

        let expected_command = Command::new(
            "multiline2",
            "multiline command".into(),
            Language::Shell,
            vec!["name"],
            "{}\necho 'Hello, $name.sh';",
        );
        let actual_command = command()
            .parse(
r#"/// multiline command
sh fn multiline2(name) {
    {}
    echo 'Hello, $name.sh';
}"#,
            )
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("multiline2", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );

        let expected_command = Command::new(
            "multiline3",
            "multiline command".into(),
            Language::Javascript,
            vec!["name"],
            "function greet() {\n    console.log('Hello, $name.js');\n}\ngreet();",
        );
        let actual_command = command()
            .parse(
r#"/// multiline command
    js fn multiline3(name) {
        function greet() {
            console.log('Hello, $name.js');
        }
        greet();
    }"#,
            )
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("multiline3", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );
    }

    #[test]
    fn runfile() {
        let expected_runfile = Runfile {
            commands: HashMap::from_iter([
                (
                    "greet",
                    Command::new(
                        "greet",
                        "Greets the user".into(),
                        Language::Bash,
                        vec!["name"],
                        "echo \"Hello, $name.sh\";",
                    ),
                ),
                (
                    "pata",
                    Command::new(
                        "pata",
                        "".into(),
                        Language::Shell,
                        vec!["name", "age"],
                        "echo \"Hello, $name.sh\";",
                    ),
                ),
            ]),
            ..Default::default()
        };
        let actual_runfile = super::runfile()
            .parse(
                r#"
            /// Greets the user
            bash fn greet(name) { 
                echo "Hello, $name.sh";
            }
            
            // This is a comment without doc
            fn pata (name age) {
                echo "Hello, $name.sh";
            }"#,
            )
            .unwrap();
        assert_eq!(
            actual_runfile, expected_runfile,
            "\nActual: {:#?}\nExpected: {:#?}",
            actual_runfile, expected_runfile
        );
    }

    #[test]
    fn subcommand() {
        let expected = Runfile {
            commands: HashMap::from_iter([(
                "greet",
                Command::new(
                    "greet",
                    "Greets the user".into(),
                    Language::Shell,
                    vec!["name"],
                    "echo \"Hello, $name.sh\";",
                ),
            )]),
            ..Default::default()
        };
        let actual = super::subcommand(super::runfile())
            .parse(
                r#"sub subcommand {
                /// Greets the user
                sh fn greet(name) { 
                    echo "Hello, $name.sh";
                }
            }"#,
            )
            .unwrap();
        let expected = ("subcommand", expected);
        assert_eq!(
            actual, expected,
            "\nActual: {:#?}\nExpected: {:#?}",
            actual, expected
        );
    }
}
