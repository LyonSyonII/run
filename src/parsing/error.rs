use crate::strlist::Str;

pub struct ParseError {
    pub msg: Str<'static>,
    pub start: usize,
    pub end: usize,
}

impl ParseError {
    pub fn new(msg: impl Into<Str<'static>>, start: usize, end: usize) -> Self {
        Self {
            msg: msg.into(),
            start,
            end,
        }
    }

    pub fn msg(&self) -> &str {
        self.msg.as_ref()
    }

    pub fn err<T>(msg: impl Into<Str<'static>>, start: usize, end: usize) -> Result<T, Self> {
        Err(Self::new(msg, start, end))
    }
}
