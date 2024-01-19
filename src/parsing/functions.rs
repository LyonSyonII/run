use crate::strlist::Str;

use super::{checkpoint::Checkpoint, error::Error};

type ParserResult<T, P> = Result<(T, P), (Error, P)>;
type ParserResultIgnore<P> = Result<P, (Error, P)>;
type ParserOk<T, P> = (T, P);
type ParserErr<P> = (Error, P);

pub trait ParseFunctions<'r, 'i>
where
    Self: Sized,
{
    fn input_raw(&self) -> &'i str;
    fn input(&self) -> &'i str {
        self.input_raw().get(self.pos()..).unwrap_or_default()
    }
    fn chars(&self) -> std::str::Chars<'i>;
    fn checkpoint(self) -> Checkpoint<'r, 'i, Self>;
    fn pos(&self) -> usize;
    fn pos_mut(&'r mut self) -> &'r mut usize;
    fn set_pos(&mut self, pos: usize);

    /// Increments the parser's position by one character `c`.
    ///
    /// If `c` is greater than the number of characters left in the parser's input, the parser's position is set to the end of the input.
    fn increment(&mut self, c: char) {
        let pos = self.pos();
        let newpos = (pos + c.len_utf8()).min(pos + self.input().len());
        self.set_pos(newpos);
    }
    /// Increments the parser's position by one character.
    ///
    /// If `c` is greater than the number of characters left in the parser's input, the parser's position stays the same
    /// and an error is returned.
    fn try_increment(&mut self, c: char) -> Result<(), ()> {
        let pos = self.pos();
        let newpos = pos + c.len_utf8();
        if newpos > pos + self.input().len() {
            Err(())
        } else {
            self.set_pos(newpos);
            Ok(())
        }
    }
    fn try_increment_bytes(&mut self, bytes: usize) -> Result<usize, usize> {
        let pos = self.pos();
        if self.input().len() >= bytes {
            self.set_pos(pos + bytes);
            Ok(pos)
        } else {
            let n = self.input().len();
            self.set_pos(pos + n);
            Err(pos)
        }
    }
    fn increment_bytes(&mut self, bytes: usize) -> usize {
        let pos = self.pos();
        let n = self.input().len().min(bytes);
        self.set_pos(pos + n);
        pos
    }
    /// Increments the parser's position by `n` characters and returns the new position.
    ///
    /// If `n` is greater than the number of characters left in the parser's input, the parser's position is set to the end of the input
    /// and an error is returned.
    ///
    /// # Examples
    /// ```
    /// let mut parser = Parser::new("123456789");
    /// parser.increment_n(3);
    /// assert_eq!(parser.pos(), 3);
    /// assert_eq!(parser.input(), "456789");
    /// ```
    fn increment_n(mut self, n: usize) -> Result<(usize, Self), (usize, Self)> {
        let n = self.input().char_indices().nth(n).map(|(i, _)| i);
        if let Some(n) = n {
            self.increment_bytes(n);
            Ok((self.pos(), self))
        } else {
            let n = self.input().len();
            self.increment_bytes(n);
            Err((self.pos(), self))
        }
    }
    /// Increments the parser's position by `n` characters and returns the new position.
    ///
    /// If `n` is greater than the number of characters left in the parser's input, the parser's position stays the same and an error is returned.
    ///
    /// # Examples
    /// ```
    /// let mut parser = Parser::new("123456789");
    /// parser.increment_n(3);
    /// assert_eq!(parser.pos(), 3);
    /// assert_eq!(parser.input(), "456789");
    /// ```
    fn try_increment_n(mut self, n: usize) -> Result<(usize, Self), Self> {
        let n = self.input().char_indices().nth(n).map(|(i, _)| i);
        if let Some(n) = n {
            self.increment_bytes(n);
            Ok((self.pos(), self))
        } else {
            Err(self)
        }
    }
    fn repeated_ignore<T>(self, f: impl Fn(Self) -> ParserResult<T, Self>) -> ParserResultIgnore<Self>
    where
    {
        self.repeated(f).map(|(_, p)| p)
    }
    fn repeated<T>(self, f: impl Fn(Self) -> ParserResult<T, Self>) -> ParserResult<Vec<T>, Self> {
        let mut consumed = Vec::new();
        // If first iteration fails, return error
        let (first, mut parser) = f(self)?;
        consumed.push(first);

        loop {
            match f(parser) {
                Ok((s, c)) => {
                    consumed.push(s);
                    parser = c
                }
                Err((_, c)) => break parser = c,
            }
        }
        Ok((consumed, parser))
    }
    fn ignore(mut self, s: impl AsRef<str>) -> ParserResultIgnore<Self> {
        let s = s.as_ref();
        if self.input().starts_with(s) {
            self.increment_bytes(s.len());
            Ok(self)
        } else {
            self.err(format!("Expected {:?}", s))
        }
    }
    fn ignore_char(mut self, c: char) -> ParserResultIgnore<Self> {
        if self.input().starts_with(c) {
            self.increment(c);
            Ok(self)
        } else {
            self.err(format!("Expected {:?}", c))
        }
    }
    /// Ignores `n` characters.
    ///
    /// If `n` is greater than the number of characters left, an error is returned.
    ///
    /// # Examples
    /// ```
    /// let mut parser = Parser::new("123456789");
    /// parser.ignore_n(3);
    /// assert_eq!(parser.pos(), 3);
    /// assert_eq!(parser.input(), "456789");
    ///
    /// parser.ignore_n(4);
    /// assert_eq!(parser.pos(), 7);
    /// assert_eq!(parser.input(), "89");
    ///
    /// assert_eq!(parser.ignore_n(3).map_err(|(e, _)| e), Err(Error::new(7, 9, "Expected 3 characters, found 2"));
    /// ```
    fn ignore_n(self, n: usize) -> ParserResultIgnore<Self> {
        match self.try_increment_n(n) {
            Ok((_, p)) => Ok(p),
            Err(p) => {
                let len = p.input().len();
                p.err(format!("Expected {n} characters, found {len}"))
            }
        }
    }
    fn skip_while(mut self, while_fn: impl Fn(char) -> bool) -> Self {
        for c in self.chars() {
            if !while_fn(c) {
                break;
            }
            self.increment(c);
        }
        self
    }
    fn skip_inline_whitespace(self) -> Self {
        self.skip_while(|c| c == ' ')
    }
    /// Consumes `n` characters.
    ///
    /// If `n` is greater than the number of characters left in the parser's input an error is returned.
    fn consume_n(self, n: usize) -> ParserResult<&'i str, Self> {
        let start = self.pos();
        println!("n: {n}");
        println!("start: {start}");
        match self.try_increment_n(n) {
            Ok((end, p)) => {
                let res = p.input_raw().get(start..end).unwrap_or_default();
                Ok((res, p))
            }
            Err(p) => {
                let len = p.input().len();
                p.err(format!("Expected {n} characters, found {len}"))
            }
        }
    }
    fn consume_while(mut self, while_fn: impl Fn(char) -> bool) -> (&'i str, Self) {
        let mut end = 0;
        for c in self.chars() {
            if !while_fn(c) {
                break;
            }
            end += c.len_utf8();
        }
        let consumed = self.input().get(..end).unwrap_or_default();
        self.increment_bytes(end);
        (consumed, self)
    }
    
    fn consume_until(self, until: impl Fn(char) -> bool) -> (&'i str, Self) {
        self.consume_while(|c| !until(c))
    }
    fn consume_until_included(mut self, until: impl Fn(char) -> bool) -> (&'i str, Self) {
        let start = self.pos();
        for c in self.chars() {
            self.increment(c);
            if until(c) {
                break;
            }
        }
        (self.input_raw().get(start..self.pos()).unwrap_or_default(), self)
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
    fn peek_char_is(self, is: impl FnOnce(Option<char>) -> bool) -> ParserResultIgnore<Self> {
        if is(self.chars().next()) {
            Ok(self)
        } else {
            self.err("Expected some char")
        }
    }

    /// Transforms [`Self`] into a [`Result::Ok`].
    fn ok<E>(self) -> Result<Self, E> {
        Ok(self)
    }
    /// Transforms [`Self`] into a [`Result::Err`] with the given message.
    fn err<T>(self, msg: impl Into<Str<'static>>) -> Result<T, (Error, Self)> {
        Err((Error::new(self.pos(), self.pos(), msg), self))
    }
}
