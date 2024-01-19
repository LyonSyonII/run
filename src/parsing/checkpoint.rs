/// Represents a checkpoint in the parser's state.
///
/// A checkpoint allows the parser to save its current position and restore it later.
/// This is useful when implementing backtracking or lookahead functionality.
///
/// # Example
///
/// ```
/// let mut parser = Parser::new("123456789");
/// let checkpoint = parser.checkpoint();
///
/// // When a checkpoint is created, it must be used instead of the parser
/// assert_eq!(checkpoint.take(5), "12345");
/// assert_eq!(checkpoint.pos(), 5);
/// assert_eq!(checkpoint.input(), "6789");
/// assert_eq!(checkpoint.rewind(), 5);
/// assert_eq!(checkpoint.pos(), 0); // The parser is rewound to the initial position
/// assert_eq!(checkpoint.input(), "123456789");
///
/// // Use `checkpoint.discard()` to discard the checkpoint and retrieve the parser
/// // This will not rewind the parser
/// let mut parser = checkpoint.discard();
/// ```
#[derive(Debug, PartialEq)]
pub struct Checkpoint<'reference, 'input, P>
where
    P: super::functions::ParseFunctions<'reference, 'input>,
{
    checkpoint: usize,
    parser: P,
    _marker: std::marker::PhantomData<&'reference mut &'input ()>,
}

impl<'r, 'i, P> Checkpoint<'r, 'i, P>
where
    P: super::functions::ParseFunctions<'r, 'i>,
{
    /// Creates a new checkpoint for the given parser.
    ///
    /// # Arguments
    ///
    /// * `parser` - A mutable reference to the parser.
    ///
    /// # Returns
    ///
    /// A new `Checkpoint` instance.
    pub fn new(parser: P) -> Self {
        Self {
            checkpoint: parser.pos(),
            parser,
            _marker: std::marker::PhantomData,
        }
    }

    /// Discards the checkpoint and returns a mutable reference to the parser.
    pub fn discard(self) -> P {
        self.parser
    }

    /// Rewinds the parser to the position stored in the checkpoint and returns the number of characters rewound.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut parser = Parser::new("123456789");
    /// let checkpoint = parser.checkpoint();
    ///
    /// // When a checkpoint is created, it must be used instead of the parser
    /// assert_eq!(checkpoint.take(5), "12345");
    /// assert_eq!(checkpoint.pos(), 5);
    /// assert_eq!(checkpoint.input(), "6789");
    /// assert_eq!(checkpoint.rewind(), 5);
    /// assert_eq!(checkpoint.pos(), 0); // The parser is rewound to the initial position
    /// assert_eq!(checkpoint.input(), "123456789");
    ///
    /// // Use `checkpoint.discard()` to discard the checkpoint and retrieve the parser
    /// // This will not rewind the parser
    /// let mut parser = checkpoint.discard();
    /// ```
    pub fn rewind(&'r mut self) -> usize {
        std::mem::replace(self.parser.pos_mut(), self.checkpoint)
    }
}

impl<'r, 'i, P> super::functions::ParseFunctions<'r, 'i> for Checkpoint<'r, 'i, P>
where
    P: super::functions::ParseFunctions<'r, 'i>,
{
    fn input_raw(&self) -> &'i str {
        self.parser.input_raw()
    }
    fn chars(&self) -> std::str::Chars<'i> {
        self.parser.chars()
    }
    fn checkpoint(self) -> super::Checkpoint<'r, 'i, Self> {
        super::Checkpoint::new(self)
    }
    fn pos(&self) -> usize {
        self.parser.pos()
    }
    fn pos_mut(&'r mut self) -> &'r mut usize {
        self.parser.pos_mut()
    }
    fn set_pos(&mut self, pos: usize) {
        self.parser.set_pos(pos);
    }
}
