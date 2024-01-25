use ariadne::Fmt as _;

const REPO: &str = "https://github.com/lyonsyonii/run";

type Start = usize;
type End = usize;
type Name = String;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Error {
    /// `(lang, start, end)`
    PParseLang(String, Start, End),
    PExpectedLangOrCmd(Start, End),
    PExpectedCmdName(Start, End),
    PExpectedArgs(Start, End),
    PExpectedOpenParen(Start, End),
    PExpectedCloseParen(Start, End),
    PExpectedBodyStart(Start, End),
    PExpectedBodyEnd(usize, Start, End),
    /// `(std::io::Error, start, end)`
    PIncludeRead(String, Name, Start, End),
    /// `(peg::ParseError, start, end)`
    PIncludeParse(String, Name, Start, End),
    PMathExpression(Start, End),
    #[default]
    Unknown,
}

impl Error {
    pub fn ariadne(
        msg: impl Into<String>,
        start: usize,
        end: usize,
        file: impl AsRef<str>,
        input: impl AsRef<str>,
        color: ariadne::Color,
    ) -> std::io::Result<()> {
        let msg = msg.into();
        let file = file.as_ref();
        let input = input.as_ref();

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

    pub fn eprint(&self, file: &str, input: &str, color: ariadne::Color) -> std::io::Result<()> {
        use format as f;
        let ariadne = |msg: &str, start: &usize, end: &usize| {
            Self::ariadne(msg, *start, *end, file, input, color)
        };

        match self {
            Error::Unknown => eprintln!("Unknown error, please report this issue on {}", REPO),
            Error::PExpectedLangOrCmd(start, end) => ariadne("Expected language or fn/cmd", start, end)?,
            Error::PExpectedCmdName(start, end) => ariadne("Expected command name", start, end)?,
            Error::PParseLang(s, start, end) => ariadne(&f!("Unknown language '{s}'; expected one of [cmd, fn, sh, shell, bash, rs, rust, py, python, js, javascript]"), start, end)?,
            Error::PExpectedArgs(start, end) => ariadne("Expected '(args)' or empty parentheses '()' after command name", start, end)?,
            Error::PExpectedOpenParen(start, end) => ariadne("Expected open parentheses '('", start, end)?,
            Error::PExpectedCloseParen(start, end) => ariadne("Expected close parentheses ')'", start, end)?,
            Error::PExpectedBodyStart(start, end) => ariadne("Expected command body start '{'", start, end)?,
            Error::PExpectedBodyEnd(count, start, end) => ariadne(&f!("Expected command body end '{}'", "}".repeat(*count)), start, end)?,
            Error::PIncludeRead(e, name, start, end) => ariadne(&f!("Failed to read included file '{name}': {e}"), start, end)?,
            Error::PIncludeParse(e, name, start, end) => ariadne(&f!("Failed to parse included file '{name}': {e}"), start, end)?,
            Error::PMathExpression(start, end) => ariadne("Failed to parse math expression", start, end)?,
        }

        Ok(())
    }

    pub fn err<T>(self) -> Result<T, Self> {
        Err(self)
    }
}
