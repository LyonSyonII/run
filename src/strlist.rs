use colored::{Color, Colorize, Style, Styles};

pub struct StrList<'a> {
    elements: Vec<std::borrow::Cow<'a, str>>,
    separator: std::borrow::Cow<'a, str>,
}

pub struct StrListSlice<'a> {
    elements: &'a [std::borrow::Cow<'a, str>],
    separator: &'a std::borrow::Cow<'a, str>,
    color: Color,
    bold: bool,
}

impl<'a> StrList<'a> {
    pub fn new(separator: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        Self {
            elements: Vec::new(),
            separator: separator.into(),
        }
    }

    pub fn append(mut self, s: impl Into<std::borrow::Cow<'a, str>>) -> Self {
        self.elements.push(s.into());
        self
    }

    pub fn except_last(&'a self) -> StrListSlice<'a> {
        let last = self.elements.len() - 1;
        StrListSlice::new(self.elements.get(..last).unwrap_or(&[]), &self.separator)
    }

    pub fn last(&'a self) -> StrListSlice<'a> {
        let last = self.elements.len() - 1;
        StrListSlice::new(
            self.elements.get(last..=last).unwrap_or(&[]),
            &self.separator,
        )
    }

    pub fn as_slice(&'a self) -> StrListSlice<'a> {
        StrListSlice::new(&self.elements, &self.separator)
    }
}

impl<'a> StrListSlice<'a> {
    fn new(
        elements: &'a [std::borrow::Cow<'a, str>],
        separator: &'a std::borrow::Cow<'a, str>,
    ) -> Self {
        Self {
            elements,
            separator,
            color: Color::White,
            bold: false,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
}

impl<'a> std::fmt::Display for StrList<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

impl<'a> std::fmt::Display for StrListSlice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.elements.iter().map(|s| {
            let mut s = s.color(self.color);
            if self.bold {
                s = s.bold()
            }
            s
        });
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for s in iter {
                write!(f, "{}{}", self.separator, s)?;
            }
        }
        Ok(())
    }
}

impl<'a, Separator, I, S> From<(Separator, I)> for StrList<'a>
where
    Separator: Into<std::borrow::Cow<'a, str>>,
    I: IntoIterator<Item = S>,
    S: Into<std::borrow::Cow<'a, str>>,
{
    fn from((separator, v): (Separator, I)) -> Self {
        Self {
            elements: v.into_iter().map(|s| s.into()).collect(),
            separator: separator.into(),
        }
    }
}
