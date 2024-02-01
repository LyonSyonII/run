use super::Str;

#[derive(Debug, Clone)]
pub struct StrList<'a> {
    elements: Vec<Str<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct StrListSlice<'a> {
    elements: &'a [Str<'a>],
}

#[allow(dead_code)]
impl<'a> StrList<'a> {
    pub fn new(separator: impl Into<Str<'a>>) -> Self {
        Self {
            elements: vec![separator.into()],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.len() <= 1
    }

    pub fn elements(&'a self) -> &'a [Str<'a>] {
        self.elements.get(1..).unwrap_or_default()
    }

    pub fn separator(&'a self) -> &'a str {
        self.elements.first().map(Str::as_ref).unwrap_or_default()
    }

    pub fn append(mut self, s: impl Into<Str<'a>>) -> Self {
        self.elements.push(s.into());
        self
    }

    pub fn extend(mut self, s: impl IntoIterator<Item = impl Into<Str<'a>>>) -> Self {
        self.elements.extend(s.into_iter().map(|s| s.into()));
        self
    }

    pub fn prepend(mut self, s: impl Into<Str<'a>>) -> Self {
        self.elements.insert(1, s.into());
        self
    }

    pub fn pop(&mut self) -> Option<Str<'a>> {
        if self.elements.len() <= 1 {
            return None;
        }
        self.elements.pop()
    }

    pub fn pop_front(&mut self) -> Option<Str<'a>> {
        if self.elements.len() <= 1 {
            return None;
        }
        Some(self.elements.remove(1))
    }

    pub fn first(&'a self) -> Option<&'a str> {
        self.elements.get(1).map(Str::as_ref)
    }

    pub fn last(&'a self) -> Option<&'a str> {
        if self.elements.len() <= 1 {
            return None;
        }
        self.elements.last().map(Str::as_ref)
    }

    pub fn except_last(&'a self) -> StrListSlice<'a> {
        let last = self.elements.len() - 1;
        StrListSlice::new(self.elements().get(..last).unwrap_or(&[]))
    }

    pub fn last_slice(&'a self) -> StrListSlice<'a> {
        let last = self.elements.len() - 1;
        if last == 0 {
            return StrListSlice::new(&[]);
        }
        StrListSlice::new(self.elements().get(last..=last).unwrap_or(&[]))
    }

    pub fn as_slice(&'a self) -> StrListSlice<'a> {
        StrListSlice::new(&self.elements)
    }
}

#[allow(dead_code)]
impl<'a> StrListSlice<'a> {
    fn new(elements: &'a [Str<'a>]) -> Self {
        Self {
            elements,
        }
    }

    pub fn first(&'a self) -> Option<&'a str> {
        self.elements.get(1).map(Str::as_ref)
    }

    pub fn last(&'a self) -> Option<&'a str> {
        if self.elements.len() <= 1 {
            return None;
        }
        self.elements.last().map(Str::as_ref)
    }

    pub fn separator(&'a self) -> &'a str {
        self.elements.first().map(Str::as_ref).unwrap_or_default()
    }

    pub fn elements(&'a self) -> &'a [Str<'a>] {
        self.elements.get(1..).unwrap_or_default()
    }
}

impl<'a> std::fmt::Display for StrList<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

impl<'a> std::fmt::Display for StrListSlice<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.elements().iter();
        let separator = self.separator();
        if let Some(first) = iter.next() {
            write!(f, "{}", first)?;
            for s in iter {
                write!(f, "{}{}", separator, s)?;
            }
        }
        Ok(())
    }
}

impl<'a, Separator, I, S> From<(Separator, I)> for StrList<'a>
where
    Separator: Into<Str<'a>>,
    I: IntoIterator<Item = S>,
    S: Into<Str<'a>>,
{
    fn from((separator, v): (Separator, I)) -> Self {
        Self {
            elements: std::iter::once(separator.into())
                .chain(v.into_iter().map(|s| s.into()))
                .collect(),
        }
    }
}

impl<'a> IntoIterator for StrList<'a> {
    type Item = Str<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(mut self) -> Self::IntoIter {
        // Remove the separator
        self.elements.remove(0);
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for StrListSlice<'a> {
    type Item = &'a Str<'a>;
    type IntoIter = std::slice::Iter<'a, Str<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.get(1..).unwrap_or_default().iter()
    }
}

impl<'a, S> std::iter::Extend<S> for StrList<'a> where S: Into<Str<'a>> {
    fn extend<T: IntoIterator<Item = S>>(&mut self, iter: T) {
        let iter = iter.into_iter().map(|s| s.into());
        self.elements.extend(iter);
    }
}