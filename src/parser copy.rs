use crate::{
    command::Command, lang::Language, runfile::Runfile, strlist::Str, utils::BoolExt as _,
};
pub use std::format as fmt;







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


    pub fn peek_char_is<'r>(
        &'r mut self,
        is: impl FnOnce(Option<char>) -> bool,
    ) -> ParserResultIgnore<'r, 'i> {
        if is(self.chars().next()) {
            Ok(self)
        } else {
            self.err("Expected some char")
        }
    }

    pub fn peek_is<'r>(
        &'r mut self,
        is: impl FnOnce(&'i str) -> bool,
    ) -> Result<&'r mut Self, &'r mut Self> {
        if is(pinput!(self)) {
            Ok(self)
        } else {
            Err(self)
        }
    }

    pub fn ignore<'r>(&'r mut self, s: &str) -> ParserResultIgnore<'r, 'i> {
        let mut c = self.checkpoint();
        
        let chars = s.chars().map(Some).chain(std::iter::once(None));
        let input = c.chars().map(Some).chain(std::iter::once(None));

        for (c1, c2) in chars.zip(input) {
            let len = match c1 {
                Some(c) => c,
                None => break,
            };
            if c1 != c2 {
                return c.rewind_err(fmt!("Expected '{}'", s))?;
            }
            c.increment(len);
        }

        Ok(c.discard())
    }

    /// Matches the given char and ignores it.
    ///
    /// Reverse of [`Parser::consume`].
    pub fn ignore_char<'r>(&'r mut self, ignore: char) -> ParserResultIgnore<'r, 'i> {
        match self.consume_char(ignore) {
            Ok((_, p)) => Ok(p),
            Err(e) => Err(e),
        }
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

    /// Consumes the given `char` if it matches the input.
    ///
    /// Rewinds if the input does not match.
    pub fn consume_char<'r>(&'r mut self, consume: char) -> ParserResult<'r, 'i, char> {
        let mut c = self.checkpoint();
        match c.chars().next() {
            Some(found) if found != consume => {
                c.rewind_err(fmt!("Expected '{consume}', found {found:?}"))
            }
            Some(found) => {
                c.increment(found);
                Ok((found, c.discard()))
            }
            None => c.rewind_err(fmt!("Expected '{consume}', found EOF")),
        }
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

    pub fn consume_until_included(&mut self, until: impl Fn(char) -> bool) -> &'i str {
        let start = self.pos;
        for c in pchars!(self) {
            self.pos += c.len_utf8();
            if until(c) {
                break;
            }
        }
        self.input.get(start..self.pos).unwrap_or_default()
    }
    
    pub fn keyword<'r>(&'r mut self, keyword: &str) -> ParserResultIgnore<'r, 'i> {
        let c = self.checkpoint();
        
        c.ignore(keyword)
            .rewind_if_err()?
            .peek_char_is(|c| match c {
                Some(c) => !c.is_alphabetic(),
                None => true,
            })
            .discard_or_rewind()
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

    /// Returns `Err` with the given error and the Checkpoint.
    pub fn err<T>(
        self,
        msg: impl Into<Str<'input>>,
    ) -> Result<T, CheckpointErr<'reference, 'input>> {
        Err((Error::new(self.checkpoint, self.pos(), msg), self))
    }

    /// Updates the checkpoint to the current parser position.
    pub fn update_checkpoint(&mut self) {
        self.checkpoint = self.pos();
    }

    /// Rewinds the parser to the last checkpoint and returns the previous position.
    pub fn rewind(&mut self) -> usize {
        std::mem::replace(&mut self.parser.pos, self.checkpoint)
    }

    /// Rewinds the parser to the last checkpoint and adds `msg` to the errors.
    ///
    /// Returns `Err` with the parser.
    pub fn rewind_err<T>(
        self,
        msg: impl Into<Str<'input>>,
    ) -> Result<T, (Error<'input>, &'reference mut Parser<'input>)> {
        let (end, parser) = self.rewind_discard();
        Err((Error::new(parser.pos, end, msg), parser))
    }

    /// Discards the `Checkpoint` and returns the `Parser` without rewinding.
    pub fn discard(self) -> &'reference mut Parser<'input> {
        let Checkpoint {
            parser,
            checkpoint: _,
        } = self;
        parser
    }

    /// Discards the `Checkpoint` and returns `Ok` with the `Parser` without rewinding.
    pub fn discard_ok<E>(self) -> Result<&'reference mut Parser<'input>, E> {
        Ok(self.discard())
    }

    /// Discards the `Checkpoint` and returns `Err` with the specified error and the `Parser` without rewinding.
    pub fn discard_err<T>(
        self,
        msg: impl Into<Str<'input>>,
    ) -> Result<T, ParserErr<'reference, 'input>> {
        Err((Error::new(self.checkpoint, self.pos(), msg), self.discard()))
    }

    /// Discards the `Checkpoint`, rewinds the `Parser` and returns it with the last position before the rewind.
    pub fn rewind_discard(mut self) -> (usize, &'reference mut Parser<'input>) {
        (self.rewind(), self.parser)
    }

    /// Returns `Ok` if the given function returns true for the next character.
    ///
    /// Does not advance the parser.
    pub fn peek_char_is(
        self,
        is: impl FnOnce(Option<char>) -> bool,
    ) -> CheckpointResultIgnore<'reference, 'input> {
        self.with_parser_ignore(|p| p.peek_char_is(is))
    }

    /// Returns `Ok` if the given function returns true for the current input.
    ///
    /// Does not advance the parser.
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
    pub fn ignore(self, s: &str) -> CheckpointResultIgnore<'reference, 'input> {
        self.with_parser_ignore(|p| p.ignore(s))
    }

    /// Ignores the given char if it matches the input.
    ///
    /// Returns `Err` if the input does not match.
    pub fn ignore_char(self, ignore: char) -> CheckpointResultIgnore<'reference, 'input> {
        self.with_parser_ignore(|p| p.ignore_char(ignore))
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

    pub fn consume_while(
        self,
        while_fn: impl Fn(char) -> bool,
    ) -> Result<(&'input str, Self), Self> {
        let s = self.parser.consume_while(while_fn);
        if s.is_empty() {
            return Err(self);
        }
        Ok((s, self))
    }

    /// Consumes characters until the given function returns true.
    ///
    /// Returns the consumed characters and the new checkpoint.
    ///
    /// Does not consume the character that returned true.
    pub fn consume_until(self, until: impl Fn(char) -> bool) -> Result<(&'input str, Self), Self> {
        self.consume_while(|c| !until(c))
    }

    /// Consumes characters until the given function returns true.
    ///
    /// Returns the consumed characters and the new checkpoint.
    ///
    /// Consumes the character that returned true.
    pub fn consume_until_included(
        self,
        until: impl Fn(char) -> bool,
    ) -> CheckpointResult<'reference, 'input, &'input str> {
        let s = self.parser.consume_until_included(until);
        if s.is_empty() {
            return self.err("");
        }
        Ok((s, self))
    }

    pub fn consume_recursive(
        self,
        recursive: impl Fn(Self) -> CheckpointResult<'reference, 'input, &'input str>,
    ) -> CheckpointResult<'reference, 'input, Vec<&'input str>> {
        let mut lines = Vec::new();
        // If first iteration fails, return the error.
        let mut result = match recursive(self) {
            Ok((s, c)) => {
                lines.push(s);
                Ok(c)
            }
            Err(e) => Err(e),
        }?;

        // Continue until an error is found.
        loop {
            match recursive(result) {
                Ok((s, c)) => {
                    lines.push(s);
                    result = c
                }
                Err((_, c)) => break result = c,
            }
        }
        Ok((lines, result))
    }

    pub fn keyword(self, keyword: &str) -> Result<Self, ParserErr<'reference, 'input>> {
        let c = self
            .with_parser_ignore(|p| p.keyword(keyword))
            .rewind_if_err()?;
        Ok(c)
    }
    
    /// Executes the given function with the parser and returns the result.
    ///
    /// Keeps the current checkpoint.
    /// 
    /// Equivalent to:
    /// ```ignore
    /// match with(parser) {
    ///     Ok((t, parser)) => Ok((
    ///         t,
    ///         Self {
    ///             parser,
    ///             checkpoint: self.checkpoint // Keep checkpoint
    ///         },
    ///     )),
    ///     Err((err, parser)) => Err((
    ///         err,
    ///         Self {
    ///             parser,
    ///             checkpoint: self.checkpoint // Keep checkpoint
    ///         },
    ///     )),
    /// }
    /// ```
    fn with_parser<T>(
        self,
        with: impl FnOnce(&'reference mut Parser<'input>) -> ParserResult<'reference, 'input, T>,
    ) -> CheckpointResult<'reference, 'input, T> {
        match with(self.parser) {
            Ok((t, parser)) => Ok((
                t,
                Self {
                    parser,
                    checkpoint: self.checkpoint,
                },
            )),
            Err((err, parser)) => Err((
                err,
                Self {
                    parser,
                    checkpoint: self.checkpoint,
                },
            )),
        }
    }

    /// Executes the given function with the parser and returns `Ok` if it was successful.
    ///
    /// Keeps the current checkpoint.
    /// 
    /// Equivalent to:
    /// ```
    /// match with(parser) {
    ///     Ok(parser) => Ok(
    ///         Parser { 
    ///             parser,
    ///             checkpoint: self.checkpoint // Keep checkpoint
    ///         },
    ///     ),
    ///     Err((err, parser)) => Err((
    ///         err,
    ///         Parser {
    ///             parser,
    ///             checkpoint: self.checkpoint // Keep checkpoint
    ///         },
    ///     )),
    /// }
    /// ```
    fn with_parser_ignore(
        self,
        with: impl FnOnce(&'reference mut Parser<'input>) -> ParserResultIgnore<'reference, 'input>,
    ) -> CheckpointResultIgnore<'reference, 'input> {
        match with(self.parser) {
            Ok(parser) => Ok(
                Self {
                    parser,
                    checkpoint: self.checkpoint,
                },
            ),
            Err((err, parser)) => Err((
                err,
                Self {
                    parser,
                    checkpoint: self.checkpoint,
                },
            )),
        }
    }
}

/* impl Drop for Checkpoint<'_, '_> {
    fn drop(&mut self) {
        self.parser.pos = self.checkpoint;
    }
} */

impl<'i> AsRef<Parser<'i>> for Checkpoint<'i, '_> {
    fn as_ref(&self) -> &Parser<'i> {
        self.parser
    }
}

impl<'i> AsMut<Parser<'i>> for Checkpoint<'i, '_> {
    fn as_mut(&mut self) -> &mut Parser<'i> {
        self.parser
    }
}

fn doc<'i, 'r: 'i>(parser: &'r mut Parser<'i>) -> Result<Vec<&'i str>, Error<'i>> {
    let (s, c) = parser
        .checkpoint()
        .consume_recursive(|c| {
            let (line, c) = c
                .skip_inline_whitespace()
                .ignore("///")?
                .skip_inline_whitespace()
                .consume_until_included(|c: char| c == '\n')?;
            Ok((line.trim(), c))
        })
        .map_err(|(e, _)| e)?;

    if let Ok(c) = c.ignore_char('\n') {
        return c
            .discard_err("Documentation comments must be adjacent to the documented item")
            .map_err(|(e, _)| e);
    };

    Ok(s)
}

pub fn runfile<'input>(input: &'input str) -> Result<Runfile<'input>, Vec<Error<'input>>> {
    let mut parser = Parser::new(input);
    let doc: Vec<&'input str> = doc(&mut parser).unwrap_or_default();
    // parser.skip_whitespace();
    Ok(Runfile::default())
}

trait ResultExt<T, E> {
    fn ignore(self) -> Result<(), E>;
    fn ignore_err(self) -> Result<T, ()>;
    fn into_ignore_tuple(self) -> Result<((), T), E>;
    fn into_ignore_tuple_err(self) -> Result<T, ((), E)>;
}
impl<T, E> ResultExt<T, E> for Result<T, E> {
    /// Converts a `Result<T, E>` into a `Result<(), E>`.
    fn ignore(self) -> Result<(), E> {
        self.map(|_| ())
    }
    /// Converts a `Result<T, E>` into a `Result<T, ()>`.
    fn ignore_err(self) -> Result<T, ()> {
        self.map_err(|_| ())
    }
    /// Converts a `Result<T, E>` into a `Result<((), T), E>`.
    fn into_ignore_tuple(self) -> Result<((), T), E> {
        self.map(|t| ((), t))
    }
    /// Converts a `Result<T, E>` into a `Result<T, ((), E)>`.
    fn into_ignore_tuple_err(self) -> Result<T, ((), E)> {
        self.map_err(|e| ((), e))
    }
}

trait ResultIgnoreOkExt<T, E> {
    fn from_ignore_tuple(self) -> Result<T, E>;
}
impl<T, E> ResultIgnoreOkExt<T, E> for Result<((), T), E> {
    /// Converts a `Result<((), T), E>` into a `Result<T, E>`.
    fn from_ignore_tuple(self) -> Result<T, E> {
        self.map(|(_, t)| t)
    }
}

trait ResultIgnoreErrExt<T, E> {
    fn from_ignore_tuple_err(self) -> Result<T, E>;
}
impl<T, E> ResultIgnoreErrExt<T, E> for Result<T, ((), E)> {
    /// Converts a `Result<((), T), E>` into a `Result<T, E>`.
    fn from_ignore_tuple_err(self) -> Result<T, E> {
        self.map_err(|(_, e)| e)
    }
}

trait ResultSameExt<T> {
    fn unwrap_ignore(self) -> T;
}

impl<T> ResultSameExt<T> for Result<T, T> {
    /// Unwraps the result, ignoring the error.
    fn unwrap_ignore(self) -> T {
        match self {
            Ok(ok) => ok,
            Err(err) => err,
        }
    }
}

trait ResultIgnoreExt<'r, 'i, T> {
    fn unwrap_ignore(self) -> T;
}

impl<'r, 'i> ResultIgnoreExt<'r, 'i, &'r mut Parser<'i>> for ParserResultIgnore<'r, 'i> {
    /// Unwraps [`ParserResultIgnore`], ignoring the error and returning the [`Parser`].
    fn unwrap_ignore(self) -> &'r mut Parser<'i> {
        match self {
            Ok(c) => c,
            Err((_, c)) => c,
        }
    }
}

impl<'r, 'i> ResultIgnoreExt<'r, 'i, Checkpoint<'i, 'r>> for CheckpointResultIgnore<'r, 'i> {
    /// Unwraps [`CheckpointResultIgnore`], ignoring the error and returning the [`Checkpoint`].
    fn unwrap_ignore(self) -> Checkpoint<'i, 'r> {
        match self {
            Ok(c) => c,
            Err((_, c)) => c,
        }
    }
}

trait CheckpointResultExt<'r, 'i> {
    type CheckpointOk;
    type CheckpointErr;
    type ParserOk;
    type ParserErr;
    
    fn rewind_if_err(self) -> Result<Self::CheckpointOk, Self::ParserErr>;
    fn discard_result(self) -> Result<Self::ParserOk, Self::ParserErr>;
    fn discard_or_rewind(self) -> Result<Self::ParserOk, Self::ParserErr>;
}

impl<'r: 'i, 'i, T> CheckpointResultExt<'r, 'i> for CheckpointResult<'r, 'i, T> {
    type CheckpointOk = CheckpointOk<'r, 'i, T>;
    type CheckpointErr = CheckpointErr<'r, 'i>;
    type ParserOk = ParserOk<'r, 'i, T>;
    type ParserErr = ParserErr<'r, 'i>;

    fn rewind_if_err(self) -> Result<Self::CheckpointOk, Self::ParserErr> {
        match self {
            Ok((t, c)) => Ok((t, c)),
            Err((e, c)) => Err((e, c.rewind_discard().1)),
        }
    }

    fn discard_or_rewind(self) -> Result<Self::ParserOk, Self::ParserErr> {
        match self {
            Ok((t, c)) => Ok((t, c.discard())),
            Err((e, c)) => Err((e, c.rewind_discard().1)),
        }
    }
    
    fn discard_result(self) -> Result<Self::ParserOk, Self::ParserErr> {
        match self {
            Ok((t, c)) => Ok((t, c.discard())),
            Err((e, c)) => Err((e, c.discard())),
        }
    }
}

impl<'r: 'i, 'i> CheckpointResultExt<'r, 'i> for CheckpointResultIgnore<'r, 'i> {
    type CheckpointOk = Checkpoint<'i, 'r>;
    type CheckpointErr = CheckpointErr<'r, 'i>;
    type ParserOk = &'r mut Parser<'i>;
    type ParserErr = ParserErr<'r, 'i>;

    fn rewind_if_err(self) -> Result<Self::CheckpointOk, Self::ParserErr> {
        match self {
            Ok(c) => Ok(c),
            Err((e, c)) => Err((e, c.rewind_discard().1)),
        }
    }
    
    fn discard_result(self) -> Result<Self::ParserOk, Self::ParserErr> {
        match self {
            Ok(c) => Ok(c.discard()),
            Err((e, c)) => Err((e, c.discard())),
        }
    }
    
    fn discard_or_rewind(self) -> Result<Self::ParserOk, Self::ParserErr> {
        match self {
            Ok(c) => Ok(c.discard()),
            Err((e, c)) => Err((e, c.rewind_discard().1)),
        }
    }
}

/* trait ErrCheckpointExt<T> {
    type Ok;
    type Err;
    type ParserErr;
    
    fn rewind_if_err(self) -> Result<T, Self::ParserErr>;
    fn discard_if_err(self) -> Result<T, Self::ParserErr>;
} */

/* impl<'r, 'i, T> ErrCheckpointExt<T>
    for Result<T, CheckpointErr<'r, 'i>>
{
    type Ok = CheckpointOk<'r, 'i, T>;
    type Err = CheckpointErr<'r, 'i>;
    type ParserErr = ParserErr<'r, 'i>;
    
    fn rewind_if_err(self) -> Result<T, Self::ParserErr> {
        match self {
            Ok(ok) => Ok(ok),
            Err((err, c)) => Err((err, c.rewind_discard().1)),
        }
    }

    fn discard_if_err(self) -> Result<T, Self::ParserErr> {
        match self {
            Ok(ok) => Ok(ok),
            Err((err, c)) => Err((err, c.discard())),
        }
    }
} */

/* /// Creates a new checkpoint.
impl<'input, 'reference> From<&'reference mut Parser<'input>> for Checkpoint<'input, 'reference> {
    fn from(parser: &'reference mut Parser<'input>) -> Self {
        Checkpoint::new(parser)
    }
} */

impl From<&'_ mut Parser<'_>> for () {
    fn from(_: &'_ mut Parser<'_>) -> Self {}
}

impl From<std::convert::Infallible> for Parser<'_> {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl From<std::convert::Infallible> for Checkpoint<'_, '_> {
    fn from(_: std::convert::Infallible) -> Self {
        unreachable!()
    }
}

impl std::fmt::Debug for Parser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field("input", &self.input)
            .field("pos", &self.pos)
            .field("input_left", &self.input())
            .field("errors", &self.errors)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignore() {
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
    fn ignore_char() {
        let p = || Parser::new("hello");
        assert!(p().ignore_char('h').is_ok());
        assert!(p().ignore_char('e').is_ok());
        assert!(p().ignore_char('l').is_ok());
        assert!(p().ignore_char('o').is_ok());
        assert!(p().ignore_char('!').is_err());

        let mut parser = Parser::new("123456789");
        assert!(parser.ignore_char('1').is_ok());
        assert!(parser.ignore_char('2').is_ok());
        assert!(parser.ignore_char('3').is_ok());
        assert!(parser.ignore_char('4').is_ok());
        assert!(parser.ignore_char('5').is_ok());
        assert!(parser.ignore_char('6').is_ok());
        assert!(parser.ignore_char('7').is_ok());
        assert!(parser.ignore_char('8').is_ok());
        assert!(parser.ignore_char('9').is_ok());
        assert!(parser.ignore_char('0').is_err());
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
            assert_eq!(doc(&mut Parser::new(i)), Ok(Vec::from(k)), "{:?}", i);
        };
        let e = |i| {
            assert_eq!(doc(&mut Parser::new(i)), Err(()), "{:?}", i);
        };

        k("/// comment", &["comment"]);
        k("/// comment\n", &["comment"]);
        k("/// comment\n/// docs\n", &["comment", "docs"]);
        e("///");
        e("/// comment\n\n/// docs");
        e("/// comment\n/// docs\n\n");

        let mut parser = Parser::new("/// comment\n/// docs\n\n///comment2\n\n///comment3");
        let k = |p: &mut Parser, k: &[&str]| {
            let p2 = p.clone();
            assert_eq!(doc(p), Ok(Vec::from(k)), "{:#?}", p2);
        };
        let e = |p: &mut Parser, span: std::ops::RangeInclusive<usize>| {
            let p2 = p.clone();
            assert_eq!(doc(p), Err(()), "{:#?}", p2);
            let expected = p.errors.last().unwrap();
            let p2 = p.clone();
            assert!(
                expected.start == *span.start() && expected.end == *span.end(),
                "{p2:#?}{expected:#?}\n{span:#?}"
            )
        };
        e(&mut parser, 0..=21);
        e(&mut parser, 22..=34);
        k(&mut parser, &["comment3"]);
        e(&mut parser, 46..=46);
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
