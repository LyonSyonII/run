use crate::{
    command::Command, lang::Language, runfile::Runfile, strlist::Str, utils::BoolExt as _,
};
pub use std::format as fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Error<'i> {
    pub start: usize,
    pub end: usize,
    msg: Str<'i>,
}

impl<'i> Error<'i> {
    pub fn new(start: usize, end: usize, msg: impl Into<Str<'i>>) -> Self {
        Self {
            start,
            end,
            msg: msg.into(),
        }
    }

    pub fn err<T>(
        start: usize,
        end: usize,
        msg: impl Into<Str<'static>>,
    ) -> std::result::Result<T, Self> {
        Err(Self {
            start,
            end,
            msg: msg.into(),
        })
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}

impl<'a> From<Error<'a>> for Vec<Error<'a>> {
    fn from(e: Error<'a>) -> Self {
        vec![e]
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Parser<'i> {
    input: &'i str,
    pos: usize,
    errors: Vec<Error<'i>>,
}

#[derive(Debug, PartialEq)]
struct Checkpoint<'input, 'reference> {
    parser: &'reference mut Parser<'input>,
    checkpoint: usize,
}

macro_rules! pinput {
    ($self:ident) => {{
        let input = $self.input;
        let pos = $self.pos;
        input.get(pos..).unwrap_or_default()
    }};
}

macro_rules! pchars {
    ($self:ident) => {
        $self.input.get($self.pos..).unwrap_or_default().chars()
    };
}

impl<'i> Parser<'i> {
    pub const fn new(input: &'i str) -> Self {
        Self {
            input,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn with_pos(input: &'i str, pos: usize) -> Self {
        Self {
            input,
            pos,
            errors: Vec::new(),
        }
    }

    pub fn input(&self) -> &str {
        self.input.get(self.pos..).unwrap_or_default()
    }

    pub fn chars(&'i self) -> std::str::Chars<'i> {
        self.input().chars()
    }

    pub fn checkpoint<'a>(&'a mut self) -> Checkpoint<'i, 'a> {
        Checkpoint::new(self)
    }

    pub fn ok<'a, E>(&'a mut self) -> Result<&'a mut Self, E> {
        Ok(self)
    }

    pub fn err<T>(mut self, msg: impl Into<Str<'i>>) -> Result<T, Self> {
        self.errors.push(Error::new(self.pos, self.pos, msg));
        Err(self)
    }

    pub fn peek_char_is<'a>(
        &'a mut self,
        is: impl FnOnce(char) -> bool,
    ) -> Result<&'a mut Self, &'a mut Self> {
        let c = self.chars().next();
        if c.is_some_and(is) {
            return Ok(self);
        } else {
            return Err(self);
        }
    }

    pub fn peek_is(
        &'i mut self,
        is: impl FnOnce(&'i str) -> bool,
    ) -> Result<&'i mut Self, &'i mut Self> {
        if is(pinput!(self)) {
            return Ok(self);
        } else {
            return Err(self);
        }
    }
    
    pub fn ignore<'a>(&'a mut self, s: &str) -> Result<&'a mut Self, &'a mut Self> {
        let mut c = self.checkpoint();

        let chars = s.chars().map(Some).chain(std::iter::once(None));
        let input = c.chars().map(Some).chain(std::iter::once(None));

        for (c1, c2) in chars.zip(input) {
            let len = match c1 {
                Some(c) => c,
                None => break,
            };
            if c1 != c2 {
                return c.err_rewind(fmt!("Expected '{}'", s));
            }
            c.increment(len);
        }

        Ok(c.discard())
    }

    pub fn ignore_char(&mut self, ignore: char) -> Result<&mut Self, &mut Self> {
        let c = self.checkpoint();
        let next = c.chars().next();
        if next != Some(ignore) {
            return c.err_rewind(fmt!("Expected '{}'", ignore));
        }
        Ok(c.discard())
    }

    /// Skips all the whitespace characters until the next newline.
    ///
    /// Does not skip the newline.
    pub fn skip_inline_whitespace(&mut self) -> &mut Self {
        let _ = self.consume_until(|c| c == '\n' || !c.is_whitespace());
        self
    }

    /// Skips all the whitespace characters specified by [`char::is_whitespace`].
    ///
    /// Includes newlines.
    pub fn skip_whitespace(&mut self) -> &mut Self {
        let _ = self.consume_while(char::is_whitespace);
        self
    }

    pub fn consume_while(&mut self, while_fn: impl Fn(char) -> bool) -> &'i str {
        let start = self.pos;
        for c in pchars!(self) {
            if !while_fn(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        self.input.get(start..self.pos).unwrap_or_default()
    }

    pub fn consume_until(&mut self, until: impl Fn(char) -> bool) -> &'i str {
        self.consume_while(|c| !until(c))
    }

    pub fn keyword<'a>(&'a mut self, keyword: &str) -> Result<&'a mut Self, &'a mut Self> {
        let c = self.checkpoint();
        println!("input left: {:?}", c.input());
        let c = c.ignore(keyword)?;
        println!("input left: {:?}", c.input());
        let c = c.peek_char_is(|c| match c {
            Some(c) => !c.is_alphabetic(),
            None => true,
        })?;
        println!("input left: {:?}", c.input());

        c.discard_ok()
    }
}

impl<'input, 'reference> Checkpoint<'input, 'reference> {
    pub fn new(parser: &'reference mut Parser<'input>) -> Self {
        let pos = parser.pos;
        Self {
            parser,
            checkpoint: pos,
        }
    }

    pub fn new_with_checkpoint(parser: &'reference mut Parser<'input>, checkpoint: usize) -> Self {
        Self { parser, checkpoint }
    }

    /// Returns the current parser position.
    pub fn pos(&self) -> usize {
        self.parser.pos
    }

    pub fn increment(&mut self, c: char) {
        self.parser.pos += c.len_utf8();
    }

    /// Returns the number of characters visited since this checkpoint.
    pub fn visited(&self) -> usize {
        self.pos() - self.checkpoint
    }

    pub fn consumed(&self) -> &'input str {
        self.parser
            .input
            .get(self.checkpoint..self.pos())
            .unwrap_or_default()
    }

    /// Returns the current available input.
    pub fn input(&self) -> &'input str {
        self.parser.input.get(self.parser.pos..).unwrap_or_default()
    }

    /// Returns the current available input as a `Chars` iterator.
    pub fn chars(&self) -> std::str::Chars<'input> {
        self.input().chars()
    }

    pub fn ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }

    pub fn err<T>(self, msg: impl Into<Str<'input>>) -> Result<T, Self> {
        self.parser
            .errors
            .push(Error::new(self.checkpoint, self.pos(), msg));
        Err(self)
    }

    /// Rewinds the parser to the last checkpoint and returns the previous position.
    pub fn rewind(&mut self) -> usize {
        std::mem::replace(&mut self.parser.pos, self.checkpoint)
    }

    /// Discards the checkpoint and returns the parser without rewinding.
    pub fn discard(self) -> &'reference mut Parser<'input> {
        let Checkpoint {
            parser,
            checkpoint: _,
        } = self;
        parser
    }

    pub fn discard_ok<E>(self) -> Result<&'reference mut Parser<'input>, E> {
        Ok(self.discard())
    }

    /// Discards the checkpoint, rewinds the parser and returns it.
    pub fn rewind_discard(mut self) -> (usize, &'reference mut Parser<'input>) {
        (self.rewind(), self.parser)
    }

    /// If `self == Err`, discards the checkpoint, rewinds the parser and adds an error to the parser.
    pub fn err_rewind<T>(
        self,
        msg: impl Into<Str<'input>>,
    ) -> Result<T, &'reference mut Parser<'input>> {
        let start = self.checkpoint;
        let (end, parser) = self.rewind_discard();
        parser.errors.push(Error::new(start, end, msg));
        Err(parser)
    }

    pub fn peek_char_is(self, is: impl FnOnce(Option<char>) -> bool) -> Result<Self, Self> {
        if is(self.chars().next()) {
            Ok(self)
        } else {
            Err(self)
        }
    }

    pub fn peek_is(self, is: impl FnOnce(&'input str) -> bool) -> Result<Self, Self> {
        let input = self.input();
        if is(input) {
            Ok(self)
        } else {
            Err(self)
        }
    }
    
    /// Ignores the given string if it matches the input.
    /// 
    /// Returns `Err` if the input does not match.
    pub fn ignore(self, s: &str) -> Result<Self, Self> {
        self.with_parser(|p| p.ignore(s))
    }
    
    /// Ignores the given char if it matches the input.
    /// 
    /// Returns `Err` if the input does not match.
    pub fn ignore_char(self, ignore: char) -> Result<Self, Self> {
        self.with_parser(|p| p.ignore_char(ignore))
    }
    
    /// Skips the given string if it matches the input.
    pub fn skip(self, s: &str) -> Self {
        self.ignore(s).unwrap_ignore()
    }
    
    /// Skips the given char if it matches the input.
    pub fn skip_char(self, c: char) -> Self {
        self.ignore_char(c).unwrap_ignore()
    }

    /// Skips all the whitespace characters until the next newline.
    ///
    /// Does not consume the newline.
    pub fn skip_inline_whitespace(self) -> Self {
        match self.consume_until(|c| c == '\n' || !c.is_whitespace()) {
            Ok((_, c)) => c,
            Err(c) => c,
        }
    }

    /// Skips all the whitespace characters [`'\n', '\r', '\t', ' '`]
    pub fn skip_whitespace(self) -> Self {
        match self.consume_while(char::is_whitespace) {
            Ok((_, c)) => c,
            Err(c) => c,
        }
    }

    pub fn consume_while(self, while_fn: impl Fn(char) -> bool) -> Result<(&'input str, Self), Self> {
        let s = self.parser.consume_while(while_fn);
        if s.is_empty() {
            return Err(self);
        }
        Ok((s, self))
    }

    pub fn consume_until(self, until: impl Fn(char) -> bool) -> Result<(&'input str, Self), Self> {
        self.consume_while(|c| !until(c))
    }

    pub fn consume_recursive(
        self,
        recursive: impl Fn(Self) -> Result<(&'input str, Self), Self>,
    ) -> Result<(Vec<&'input str>, Self), Self> {
        
        let mut lines = Vec::new();
        println!("starting recursion: {:?}", self);
        // If first iteration fails, return the error.
        let mut result = match recursive(self) {
            Ok((s, c)) => {
                lines.push(s);
                Ok(c) 
            },
            Err(e) => Err(e),
        }?;
        
        // Continue until an error is found.
        loop {
            println!("iteration: {:?}", result);
            match recursive(result) {
                Ok((s, c)) => { 
                    lines.push(s);
                    result = c 
                },
                Err(e) => break result = e,
            }
        };
        println!("result: {:?}", result);
        Ok((lines, result))
    }

    pub fn keyword(self, keyword: &str) -> Result<Self, &'reference mut Parser<'input>> {
        let c = self.with_parser(|p| p.keyword(keyword))?;
        Ok(c)
    }

    /// Executes the given function with the parser and returns the result.
    ///
    /// Keeps the current checkpoint and advances the parser even if it fails.
    fn with_parser(
        self,
        with: impl FnOnce(
            &'reference mut Parser<'input>,
        )
            -> Result<&'reference mut Parser<'input>, &'reference mut Parser<'input>>,
    ) -> Result<Self, Self> {
        match with(self.parser) {
            Ok(ok) => {
                Ok(Self {
                    parser: ok,
                    checkpoint: self.checkpoint,
                })
            },
            Err(err) => {
                Err(Self {
                    parser: err,
                    checkpoint: self.checkpoint,
                })
            },
        }
    }
}

/* impl Drop for Checkpoint<'_, '_> {
    fn drop(&mut self) {
        self.parser.pos = self.checkpoint;
    }
} */

impl<'input> AsRef<Parser<'input>> for Checkpoint<'input, '_> {
    fn as_ref(&self) -> &Parser<'input> {
        self.parser
    }
}

impl<'input> AsMut<Parser<'input>> for Checkpoint<'input, '_> {
    fn as_mut(&mut self) -> &mut Parser<'input> {
        self.parser
    }
}

fn doc<'input>(parser: &mut Parser<'input>) -> Result<Vec<&'input str>, ()> {
    let (s, _) = parser.checkpoint().consume_recursive(|c| {
        let (line, c) = c.skip_inline_whitespace()
            .ignore("///")?
            .skip_inline_whitespace()
            .consume_until(|c: char| c == '\n')?;
        Ok((line, c.skip_char('\n')))
    })?;
    Ok(s)
}

pub fn runfile<'input>(input: &'input str) -> Result<Runfile<'input>, Vec<Error<'input>>> {
    let mut parser: Parser<'input> = Parser::new(input);
    let doc: Vec<&'input str> = doc(&mut parser).unwrap_or_default();
    parser.skip_whitespace();
    Ok(Runfile::default())
}

trait ResultExt<T, E> {
    fn ignore(self) -> Result<(), E>;
    fn ignore_err(self) -> Result<T, ()>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn ignore(self) -> Result<(), E> {
        self.map(|_| ())
    }

    fn ignore_err(self) -> Result<T, ()> {
        self.map_err(|_| ())
    }
}

trait ResultSameExt<T> {
    fn unwrap_ignore(self) -> T;
}

impl <T> ResultSameExt<T> for Result<T, T> {
    /// Unwraps the result, ignoring the error.
    fn unwrap_ignore(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => err,
        }
    }
}

trait ErrCheckpointExt<'input, 'reference, T> {
    type Checkpoint;
    type Parser;

    fn rewind_if_err(self) -> Result<T, Self::Parser>;
}

impl<'input, 'reference, T> ErrCheckpointExt<'input, 'reference, T>
    for Result<T, Checkpoint<'input, 'reference>>
{
    type Checkpoint = Checkpoint<'input, 'reference>;
    type Parser = &'reference mut Parser<'input>;

    fn rewind_if_err(self) -> Result<T, Self::Parser> {
        match self {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.rewind_discard().1),
        }
    }
}

/* /// Creates a new checkpoint.
impl<'input, 'reference> From<&'reference mut Parser<'input>> for Checkpoint<'input, 'reference> {
    fn from(parser: &'reference mut Parser<'input>) -> Self {
        Checkpoint::new(parser)
    }
} */

/// Rewinds the parser to the last checkpoint.
impl<'input, 'reference> From<Checkpoint<'input, 'reference>> for &'reference mut Parser<'input> {
    fn from(c: Checkpoint<'input, 'reference>) -> Self {
        c.rewind_discard().1
    }
}

impl From<Checkpoint<'_, '_>> for () {
    fn from(c: Checkpoint<'_, '_>) -> Self {
        c.rewind_discard();
    }
}

impl From<&'_ mut Parser<'_>> for () {
    fn from(_: &'_ mut Parser<'_>) -> Self {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with() {
        let p = || Parser::new("hello");
        assert!(p().ignore("").is_ok());
        assert!(p().ignore("h").is_ok());
        assert!(p().ignore("he").is_ok());
        assert!(p().ignore("hel").is_ok());
        assert!(p().ignore("hell").is_ok());
        assert!(p().ignore("hello").is_ok());
        assert!(p().ignore("hello!").is_err());

        let mut parser = Parser::new("123456789");
        assert!(parser.ignore("1").is_ok());
        assert!(parser.ignore("23").is_ok());
        assert!(parser.ignore("456").is_ok());
        assert!(parser.ignore("111").is_err());
        assert!(parser.ignore("788").is_err());
        assert!(parser.ignore("7890").is_err());
        assert!(parser.ignore("789").is_ok());
    }

    #[test]
    fn keyword() {
        let k = |i, k| {
            assert!(
                Parser::new(i).keyword(k).is_ok(),
                "{k:?} is NOT a keyword of {i:?}"
            );
        };
        let e = |i, k| {
            assert!(
                Parser::new(i).keyword(k).is_err(),
                "{k:?} IS a keyword of {i:?}"
            );
        };
        k("if", "if");
        k("if ", "if");
        k("if(", "if");
        k("if a", "if");
        e("ifa", "if");
        e("rifa", "if");

        let input = "cmd";
        let mut parser = Parser::new(input);
        assert!(parser.keyword("c").is_err());
        assert!(parser.keyword("cm").is_err());
        assert!(parser.keyword("cmdd").is_err());
        assert!(parser.keyword("cmd").is_ok());
    }

    #[test]
    fn doc() {
        use super::doc;
        let k = |i, k: &[&str]| {
            assert_eq!(doc(&mut Parser::new(i)), Ok(Vec::from(k)));
        };
        let e = |i| {
            assert_eq!(doc(&mut Parser::new(i)), Err(()));
        };

        k("/// hola", &["hola"]);
        k("/// hola\n", &["hola"]);
        k("/// hola\n/// patata\n", &["hola\npatata"]);
        e("///");
        e("/// hola\n\n/// patata");
        e("/// hola\n/// patata\n\n");
    }
}

/*
#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::{command::Command, lang::Language, runfile::Runfile};

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
 */
