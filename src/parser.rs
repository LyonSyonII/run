use crate::{
    runner::{Command, Language},
    Goodbye, Runfile,
};
use std::collections::HashMap;
pub use std::format as fmt;
use chumsky::prelude::*;

type Error<'i> = extra::Err<Rich<'i, char>>;
type Parsed<'i, T> = Boxed<'i, 'i, &'i str, T, Error<'i>>;

trait ParserExt<'i>: Parser<'i, &'i str, &'i str, Error<'i>>
where
    Self: Sized,
{
    fn expect(
        self,
        msg: &'static str,
    ) -> chumsky::combinator::MapErr<Self, impl Fn(Rich<'i, char>) -> Rich<'i, char>> {
        let closure = move |e: Rich<'i, char>| Rich::custom(*e.span(), msg);
        self.map_err(closure)
    }
}
impl<'i, T> ParserExt<'i> for chumsky::text::Padded<T> where
    T: Parser<'i, &'i str, &'i str, Error<'i>>
{
}

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
    just("#")
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
    text::keyword("fn")
        .to(Language::Bash)
        .or(text::ident()
            .try_map(|s: &str, span| s.parse::<Language>().map_err(|e| Rich::custom(span, e)))
            .then_ignore(
                text::keyword("fn")
                    .padded()
                    .map_err(|e| error(e, "expected 'fn'")),
            ))
        .boxed()
}

fn args<'i>() -> Parsed<'i, Vec<&'i str>> {
    text::ident()
        .separated_by(text::whitespace().at_least(1))
        .allow_trailing()
        .collect()
        .delimited_by(
            just('(').padded(),
            just(')').map_err(|e| error(e, "expected ')'")),
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
        .then(args().padded())
        .map(|((lang, name), args)| (lang, name, args))
        .boxed()
}

fn command<'i>() -> Parsed<'i, (&'i str, Command<'i>)> {
    doc()
        .padded()
        .then(signature())
        .then(body().padded())
        .map(|((doc, (lang, name, args)), script)| {
            (name, Command::new(name, doc, lang, args, script))
        })
        .boxed()
}

fn commands<'i>() -> Parsed<'i, HashMap<&'i str, Command<'i>>> {
    command()
        .padded()
        .repeated()
        .collect::<HashMap<&'i str, Command<'i>>>()
        .boxed()
}

fn include<'i>() -> Parsed<'i, (&'i str, Runfile<'i>)> {
    just("in")
        .padded()
        .ignore_then(string())
        .try_map(|path, span| {
            // TODO: Remove leak (Use Cow?)
            let file = std::fs::read_to_string(path).map_err(|e| Rich::custom(span, e))?.leak();
            let runfile = runfile().parse(file).into_result().map_err(|e| {
                let errors = e
                    .into_iter()
                    .map(|e| e.to_string())
                    .fold(fmt!("include {path} has errors:"), |acc, s| {
                        fmt!("{acc}\n{s}")
                    });
                Rich::custom(span, errors)
            })?;
            Ok((path, runfile))
        })
        .boxed()
}

pub fn runfile<'i>() -> Parsed<'i, Runfile<'i>> {
    enum Results<'i> {
        Command((&'i str, Command<'i>)),
        Include((&'i str, Runfile<'i>)),
    }
    
    choice((
        command().map(Results::Command),
        include().map(Results::Include)
    ))
    .repeated()
    .collect::<Vec<Results<'i>>>()
    .map(|results| {
        results.into_iter().fold(Runfile::default(), |mut acc, new| {
            match new {
                Results::Command((name, cmd)) => {
                    acc.commands.insert(name, cmd);
                    acc
                }
                Results::Include((path, include)) => {
                    acc.commands.extend(include.commands.clone());
                    acc.includes.insert(path, include);
                    acc
                }
            }
        })
        })
    .boxed()

    /*     let command = command()
        .padded()
        .repeated()
        .collect::<HashMap<&'i str, Command<'i>>>()
        .map(|commands| Runfile { commands });
    let include = include()
        .padded(); */
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{
        runner::{Command, Language},
        Runfile,
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
            .unwrap();
        assert_eq!(
            actual_command,
            ("greet", expected_command.clone()),
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
            .unwrap();
        assert_eq!(
            actual_command,
            ("greet", expected_command.clone()),
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
            includes: Default::default(),
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
}
