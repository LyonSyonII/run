use crate::strlist::Str;

pub type Error = ParseError<'static>;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError<'i> {
    pub start: usize,
    pub end: usize,
    msg: Str<'i>,
}

impl<'i> ParseError<'i> {
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

impl<'i> From<ParseError<'i>> for Vec<ParseError<'i>> {
    fn from(e: ParseError<'i>) -> Self {
        vec![e]
    }
}
