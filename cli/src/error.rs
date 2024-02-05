use ariadne::Fmt as _;

use crate::fmt::Str;

const REPO: &str = "https://github.com/lyonsyonii/run";

type Start = usize;
type End = usize;
type Name = String;

/// Errors starting with 'P' are parsing errors.
///
/// Errors starting with 'R' are run errors.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, thiserror::Error)]
pub enum Error {
    #[error("Unknown language '{0}'; expected one of [cmd, fn, sh, shell, bash, rs, rust, py, python, js, javascript]")]
    PParseLang(String, Start, End),

    #[error("Expected language or cmd")]
    PExpectedLangOrCmd(Start, End),

    #[error("Expected command name")]
    PExpectedCmdName(Start, End),

    #[error("Expected '(args)' or empty parentheses '()' after command name")]
    PExpectedArgs(Start, End),

    #[error("Expected open parentheses '('")]
    PExpectedOpenParen(Start, End),

    #[error("Expected close parentheses ')'")]
    PExpectedCloseParen(Start, End),

    #[error("Expected command body start '{{'")]
    PExpectedBodyStart(Start, End),

    #[error("Expected command body end '{}'", "}".repeat(*.0))]
    PExpectedBodyEnd(usize, Start, End),

    #[error("Failed to read included file '{1}': {0}")]
    PIncludeRead(String, Name, Start, End),

    #[error("Failed to parse included file '{1}': {0}")]
    PIncludeParse(String, Name, Start, End),

    #[error("Failed to parse math expression")]
    PMathExpression(Start, End),

    #[error("{0}")]
    Custom(Str<'static>, Start, End),

    #[default]
    #[error("Unknown error, please report this issue on {REPO}")]
    Unknown,
}

impl Error {
    /// Prints the error to stderr with ariadne.
    pub fn ariadne(
        &self,
        file: impl AsRef<str>,
        input: impl AsRef<str>,
        color: ariadne::Color,
    ) -> std::io::Result<()> {
        let msg = self.to_string();
        let (start, end) = self.span();

        Self::ariadne_with_msg(msg, start, end, file, input, color)
    }
    /// Prints the specified message to stderr with ariadne.
    pub fn ariadne_with_msg(
        msg: impl std::fmt::Display,
        start: usize,
        end: usize,
        file: impl AsRef<str>,
        input: impl AsRef<str>,
        color: ariadne::Color,
    ) -> std::io::Result<()> {
        let file = file.as_ref();
        let input = input.as_ref();
        let msg = msg.to_string();

        ariadne::Report::build(ariadne::ReportKind::Error, file, start)
            .with_message(&msg)
            .with_label(
                ariadne::Label::new((file, start..end))
                    .with_message(msg.fg(ariadne::Color::Red))
                    .with_color(color),
            )
            .finish()
            .eprint((file, ariadne::Source::from(input)))?;

        Ok(())
    }

    pub fn span(&self) -> (usize, usize) {
        match self {
            Error::PExpectedLangOrCmd(start, end) => (*start, *end),
            Error::PExpectedCmdName(start, end) => (*start, *end),
            Error::PExpectedArgs(start, end) => (*start, *end),
            Error::PExpectedOpenParen(start, end) => (*start, *end),
            Error::PExpectedCloseParen(start, end) => (*start, *end),
            Error::PExpectedBodyStart(start, end) => (*start, *end),
            Error::PExpectedBodyEnd(_, start, end) => (*start, *end),
            Error::PIncludeRead(_, _, start, end) => (*start, *end),
            Error::PIncludeParse(_, _, start, end) => (*start, *end),
            Error::PMathExpression(start, end) => (*start, *end),
            Error::PParseLang(_, start, end) => (*start, *end),
            Error::Custom(_, start, end) => (*start, *end),
            Error::Unknown => (0, 0),
        }
    }

    pub fn err<T>(self) -> Result<T, Self> {
        Err(self)
    }
}
