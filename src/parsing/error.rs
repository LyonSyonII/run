use crate::strlist::Str;

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

impl<'i> From<Error<'i>> for Vec<Error<'i>> {
    fn from(e: Error<'i>) -> Self {
        vec![e]
    }
}
