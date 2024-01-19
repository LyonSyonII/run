use self::{checkpoint::Checkpoint, error::Error, functions::ParseFunctions as _, parser::Parser};

pub mod checkpoint;
pub mod error;
pub mod functions;
pub mod parser;
mod tests;

pub fn runfile(input: &str) -> Result<crate::runfile::Runfile, Vec<Error>> {
    let mut parser = Parser::new(input);

    Ok(crate::runfile::Runfile::default())
}

pub fn doc<'r, 'i>(parser: &'r mut Parser<'i>) -> Result<Vec<&'i str>, Error> {
    let (lines, _) = parser.repeated(|parser| {
        let (line, parser) = parser
            .skip_inline_whitespace()
            .ignore("///")?
            .skip_inline_whitespace()
            .consume_until(|c| c == '\n');
        Ok((line, parser.ignore_n(1)?))
    }).map_err(|(p, _)| p)?;
    Ok(lines)
}
