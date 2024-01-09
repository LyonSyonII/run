pub struct StrList<'a> {
    elements: Vec<std::borrow::Cow<'a, str>>,
    separator: std::borrow::Cow<'a, str>,
}

pub struct StrListSlice<'a> {
    elements: &'a [std::borrow::Cow<'a, str>],
    separator: &'a std::borrow::Cow<'a, str>,
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

    pub fn last(&self) -> &std::borrow::Cow<'a, str> {
        self.elements
            .last()
            .unwrap_or(&std::borrow::Cow::Borrowed(""))
    }

    pub fn except_last(&'a self) -> StrListSlice<'a> {
        let last = self.elements.len() - 1;
        StrListSlice {
            elements: self.elements.get(..last).unwrap_or(&[]),
            separator: &self.separator,
        }
    }

    pub fn last_slice(&'a self) -> StrListSlice<'a> {
        let last = self.elements.len() - 1;
        StrListSlice {
            elements: self.elements.get(last..=last).unwrap_or(&[]),
            separator: &self.separator,
        }
    }

    pub fn as_slice(&'a self) -> StrListSlice<'a> {
        StrListSlice {
            elements: &self.elements,
            separator: &self.separator,
        }
    }
}

impl<'a> std::fmt::Display for StrList<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.elements.iter();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for s in iter {
                write!(f, "{}{}", self.separator, s)?;
            }
        }
        Ok(())
    }
}

impl<'a> std::fmt::Display for StrListSlice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.elements.iter();
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
