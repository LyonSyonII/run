use crate::{
    command::{Command, Language},
    runfile::Runfile,
};
use chumsky::prelude::*;
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

fn doc<'i>() -> Parsed<'i, String> {
    text::inline_whitespace() // indentation
        .ignore_then(just('#'))
        .ignore_then(
            any()
                .and_is(text::newline().not())
                .repeated()
                .to_slice()
                .map(|s: &str| s.trim()),
        )
        .separated_by(text::newline())
        .allow_trailing()
        .collect::<Vec<&'i str>>()
        .map(|v| v.join("\n"))
        .boxed()
}

fn language_fn<'i>() -> Parsed<'i, Language> {
    let cmd = choice((text::keyword("fn"), text::keyword("cmd")));

    cmd.clone()
        .to(Language::Bash)
        .or(text::ident()
            .try_map(|s: &str, span| s.parse::<Language>().map_err(|e| Rich::custom(span, e)))
            .then_ignore(cmd.padded().map_err(|e| error(e, "expected 'fn' or 'cmd'"))))
        .boxed()
}

fn args<'i>() -> Parsed<'i, Vec<&'i str>> {
    text::ident()
        .separated_by(text::whitespace().at_least(1))
        .allow_trailing()
        .collect()
        .delimited_by(
            just('(').padded().expect("expected '(' before arguments"),
            just(')').expect("expected ')' after arguments"),
        )
        .boxed()
}

fn body<'i>() -> Parsed<'i, &'i str> {
    // ([^ '{' '}'] / "{" body() "}")*

    let body = recursive(|body| {
        choice((
            none_of("{}").to_slice(),
            just('{').then(body).then(just('}')).to_slice(),
        ))
        .repeated()
        // .lazy()
        .to_slice()
    });

    just('{')
        .ignore_then(body)
        .then_ignore(just('}'))
        .map(|b: &str| b.trim())
        .boxed()
}

fn signature<'i>() -> Parsed<'i, (Language, &'i str, Vec<&'i str>)> {
    language_fn()
        .padded()
        .then(text::ident().padded().expect("expected command name"))
        .then(args())
        .map(|((lang, name), args)| (lang, name, args))
        .boxed()
}

fn command<'i>() -> Parsed<'i, (&'i str, Command<'i>)> {
    doc()
        .then_ignore(
            just('\n')
                .not()
                .expect("documentation must be adjacent to a command"),
        )
        .then(signature().padded())
        .then(body().padded().expect("expected command body"))
        .map(|((doc, (lang, name, args)), script)| {
            (name, Command::new(name, doc, lang, args, script))
        })
        .padded()
        .boxed()
}

fn include<'i>() -> Parsed<'i, (&'i str, Runfile<'i>)> {
    text::keyword("in")
        .padded()
        .ignore_then(string().expect("expected include path"))
        .try_map(|path, span| {
            // TODO: Remove leak (Use Cow?)
            let file = std::fs::read_to_string(path)
                .map_err(|e| Rich::custom(span, e))?
                .leak();
            let runfile = runfile().parse(file).into_result().map_err(|e| {
                let errors = e
                    .into_iter()
                    .map(|e| fmt!("{e:?}"))
                    .fold(fmt!("include {path} has errors:"), |acc, s| {
                        fmt!("{acc}\n{s}")
                    });
                Rich::custom(span, errors)
            })?;
            Ok((path, runfile))
        })
        .boxed()
}

fn subcommand<'i>(runfile: Parsed<'i, Runfile<'i>>) -> Parsed<'i, (&'i str, Runfile<'i>)> {
    doc()
        .padded()
        .then(text::keyword("sub").expect("expected 'sub'"))
        .padded()
        .ignore_then(text::ident().expect("expected subcommand name"))
        .padded()
        .then_ignore(just('{').expect("expected '{'"))
        .then(runfile)
        .then_ignore(just('}').expect("expected '}'"))
        .boxed()
}

pub fn runfile<'i>() -> Parsed<'i, Runfile<'i>> {
    enum Results<'i> {
        Command((&'i str, Command<'i>)),
        Subcommand((&'i str, Runfile<'i>)),
        Include((&'i str, Runfile<'i>)),
    }

    recursive(|runfile| {
        choice((
            include().map(Results::Include),
            subcommand(runfile.boxed()).map(Results::Subcommand),
            command().map(Results::Command),
        ))
        .repeated()
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
                        acc.commands.extend(include.commands.clone());
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
}
impl<'i, P, T> ParserExt<'i, T> for P where P: Parser<'i, &'i str, T, Error<'i>> {}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        command::{Command, Language},
        runfile::Runfile,
    };
    use chumsky::Parser as _;

    #[test]
    fn doc() {
        use super::doc;
        assert_eq!(doc().parse("#").unwrap(), "");
        assert_eq!(doc().parse("# hola").unwrap(), "hola");
        assert_eq!(doc().parse("# hola\n").unwrap(), "hola");
        assert_eq!(doc().parse("# hola\n# patata\n").unwrap(), "hola\npatata");
        assert!(doc().parse("# hola\n\n# patata").into_result().is_err());
    }

    #[test]
    fn language() {
        use super::language_fn;
        assert_eq!(language_fn().parse("bash fn").unwrap(), Language::Bash);
        assert_eq!(language_fn().parse("fn").unwrap(), Language::Bash);
        assert!(language_fn().parse("bas fn").into_result().is_err());
        assert!(language_fn().parse("bash").into_result().is_err());
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

    #[test]
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
    }

    #[test]
    fn signature() {
        use super::signature;
        let expected_signature = (Language::Bash, "greet", vec!["name"]);
        let actual_signature = signature().parse("sh fn greet(name)").unwrap();
        assert_eq!(actual_signature, expected_signature);

        let expected_signature = (Language::Bash, "pata", vec!["name", "age"]);
        let actual_signature = signature().parse("fn pata (name age)").unwrap();
        assert_eq!(actual_signature, expected_signature);
    }

    #[test]
    fn command() {
        use super::command;
        let expected_command = Command::new(
            "greet",
            "Greets the user".into(),
            Language::Bash,
            vec!["name"],
            "echo 'Hello, $name.sh';",
        );
        let actual_command = command()
            .parse("# Greets the user\nsh fn greet(name) { echo 'Hello, $name.sh'; }")
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("greet", expected_command.clone())),
            "Actual: {:#?}\nExpected: {:#?}",
            actual_command,
            expected_command
        );

        let actual_command = command()
            .parse(
                r#"
                # Greets the user
                sh fn greet(name) { 
                    echo 'Hello, $name.sh';
                }"#,
            )
            .into_result();
        assert_eq!(
            actual_command,
            Ok(("greet", expected_command.clone())),
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
                        "Usage: run pata <NAME> <AGE>".into(),
                        Language::Bash,
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
            # Greets the user
            sh fn greet(name) { 
                echo "Hello, $name.sh";
            }
            
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
                    Language::Bash,
                    vec!["name"],
                    "echo \"Hello, $name.sh\";",
                ),
            )]),
            ..Default::default()
        };
        let actual = super::subcommand(super::runfile())
            .parse(
                r#"
            sub subcommand {
                # Greets the user
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
