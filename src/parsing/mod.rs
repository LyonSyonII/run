use self::{parser::Parser, error::Error, checkpoint::Checkpoint, functions::ParseFunctions as _};

pub mod checkpoint;
pub mod error;
pub mod functions;
pub mod parser;
mod tests;

type ParserResult<'r, 'i, T> = Result<ParserOk<'r, 'i, T>, ParserErr<'r, 'i>>;
type ParserResultIgnore<'r, 'i> = Result<&'r mut Parser<'i>, ParserErr<'r, 'i>>;
type ParserOk<'r, 'i, T> = (T, &'r mut Parser<'i>);
type ParserErr<'r, 'i> = (Error<'i>, &'r mut Parser<'i>);
type CheckpointResult<'r, 'i, T, P> = Result<CheckpointOk<'r, 'i, T, P>, CheckpointErr<'r, 'i, P>>;
type CheckpointResultIgnore<'r, 'i, P> = Result<Checkpoint<'r, 'i, P>, CheckpointErr<'r, 'i, P>>;
type CheckpointOk<'r, 'i, T, P> = (T, Checkpoint<'r, 'i, P>);
type CheckpointErr<'r, 'i, P> = (Error<'i>, Checkpoint<'r, 'i, P>);

pub fn runfile(input: &str) -> Result<crate::runfile::Runfile, Vec<Error>> {
    let mut parser = Parser::new(input);

    Ok(crate::runfile::Runfile::default())
}

pub fn doc<'r, 'i>(parser: &'r mut Parser<'i>) -> Result<Vec<&'i str>, Error<'i>> {
    let c = parser.checkpoint();
    let i = c.discard().input();
    Ok(vec![i])
}
