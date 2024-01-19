use crate::strlist::Str;

pub struct Error {
    msg: Str<'static>,
    start: usize,
    end: usize,
}