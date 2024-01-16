use crate::strlist::Str;

use super::{error::Error, checkpoint::Checkpoint};

type ParserResult<'r, 'i, T, P> = Result<(T, P), (Error<'i>, P)>;
type ParserResultIgnore<'r, 'i, P> = Result<P, (Error<'i>, P)>;
type ParserOk<'r, 'i, T, P> = (T, P);
type ParserErr<'r, 'i, P> = (Error<'i>, P);

pub trait ParseFunctions<'r, 'i>
where
    Self: Sized,
{
    fn input(&self) -> &'i str;
    fn chars(&self) -> std::str::Chars<'i>;
    fn checkpoint(self) -> Checkpoint<'r, 'i, Self>;
    fn pos(&self) -> usize;
    fn pos_mut(&'r mut self) -> &'r mut usize;
    fn set_pos(&mut self, pos: usize);
    fn increment(mut self, c: char) -> Self {
        let pos = self.pos();
        let newpos = (pos + c.len_utf8()).min(self.input().len());
        self.set_pos(pos + c.len_utf8());
        self
    }
    /// Increments the parser's position by `n` characters and returns the new position.
    ///
    /// If `n` is greater than the number of characters left in the parser's input, the parser's position is set to the end of the input.
    ///
    /// # Examples
    /// ```
    /// let mut parser = Parser::new("123456789");
    /// parser.increment_n(3);
    /// assert_eq!(parser.pos(), 3);
    /// assert_eq!(parser.input(), "456789");
    /// ```
    fn increment_n(mut self, n: usize) -> (usize, Self) {
        let end = self
            .input()
            .char_indices()
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or(self.input().len());
        let pos = self.pos()+end;
        self.set_pos(pos);
        (pos, self)
    }
    /// Transforms [`Self`] into a [`Result::Ok`].
    fn ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }
    /// Transforms [`Self`] into a [`Result::Err`] with the given message.
    fn err<T>(self, msg: impl Into<Str<'i>>) -> Result<T, (Error<'i>, Self)> {
        Err((Error::new(self.pos(), self.pos(), msg), self))
    }
    fn ignore_n(self, n: usize) -> ParserResultIgnore<'r, 'i, Self> {
        let len = self.input().len();
        let pos = self.pos();
        if pos + n > len {
            self.err(format!("Expected {n} characters, found {}", len - pos))
        } else {
            Ok(self.increment_n(n).1)
        }
    }
    /// Consumes `n` characters.
    ///
    /// If `n` is greater than the number of characters left in the parser's input, only the remaining characters are consumed.
    fn consume_n(self, n: usize) -> (&'i str, Self) {
        let (end, p) = self.increment_n(n);
        let res = p.input().get(..end).unwrap_or_default();
        (res, p)
    }
    /// Checks if the next character in the parser's input satisfies a given condition.
    ///
    /// This function takes a closure `is` that accepts an `Option<char>` and returns a `bool`.<br>
    /// The closure is applied to the next character in the parser's input.<br>
    /// If the closure returns `true`, this function returns the parser itself wrapped in an `Ok`.<br>
    /// If the closure returns `false`, this function returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut parser = Parser::new("abc");
    /// assert!(parser.peek_char_is(|c| c == Some('a')).is_ok());
    /// assert!(parser.peek_char_is(|c| c.is_some_and(char::is_alphabetic)).is_ok());
    /// ```
    fn peek_char_is(
        self,
        is: impl FnOnce(Option<char>) -> bool,
    ) -> ParserResultIgnore<'r, 'i, Self> {
        if is(self.chars().next()) {
            Ok(self)
        } else {
            self.err("Expected some char")
        }
    }
}
