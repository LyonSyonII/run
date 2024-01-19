use crate::strlist::Str;

use super::{checkpoint::Checkpoint, error::Error};

/// A parser for a given input.
///
/// Keeps track of the current position in the input, and can rewind to a previous position if an error occurs.
///
/// Rewinding must be handled explicitely by the user, using [`Parser::checkpoint`], [`Checkpoint::discard`] and [`Checkpoint::rewind`].
#[derive(Clone, PartialEq, Debug)]
pub struct Parser<'i> {
    pub(super) input: &'i str,
    pub(super) pos: usize,
    pub(super) errors: Vec<Error>,
}

impl<'i> Parser<'i> {
    /// Create a new [`Parser`] with the given input.
    pub const fn new(input: &'i str) -> Self {
        Self {
            input,
            pos: 0,
            errors: Vec::new(),
        }
    }
    /// Create a new [`Parser`] with the given input and position.
    pub fn with_pos(input: &'i str, pos: usize) -> Self {
        Self {
            input,
            pos,
            errors: Vec::new(),
        }
    }
    /// Returns the current input of the parser.
    pub fn input(&self) -> &'i str {
        self.input.get(self.pos..).unwrap_or_default()
    }
    /// Returns an iterator over the characters of the current input.
    pub fn chars(&'i self) -> std::str::Chars<'i> {
        self.input().chars()
    }
    /*     pub fn checkpoint<'r>(&'r mut self) -> Checkpoint<'r, 'i, &'r mut Self> {
        Checkpoint::new(self)
    } */
}

impl<'r, 'i> super::functions::ParseFunctions<'r, 'i> for &'r mut Parser<'i> {
    fn input_raw(&self) -> &'i str {
        self.input
    }
    fn chars(&self) -> std::str::Chars<'i> {
        self.input().chars()
    }
    /// Creates a new checkpoint and blocks usage of the parser until the checkpoint is either discarded or rewinded.
    fn checkpoint(self) -> super::Checkpoint<'r, 'i, Self> {
        super::Checkpoint::new(self)
    }
    fn pos(&self) -> usize {
        self.pos
    }
    fn pos_mut(&'r mut self) -> &'r mut usize {
        &mut self.pos
    }
    fn set_pos(&mut self, pos: usize) {
        self.pos = pos;
    }
}
