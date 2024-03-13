use super::Str;

pub type StrList<'a> = FmtList<&'static str, Str<'a>>;
pub type StrListSlice<'a> = FmtListSlice<'a, &'static str, Str<'a>>;

#[derive(Debug, Clone)]
pub struct FmtList<S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    separator: S,
    elements: Vec<D>,
}

#[derive(Debug, Clone, Copy)]
pub struct FmtListSlice<'a, S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
    S: ?Sized,
{
    separator: &'a S,
    elements: &'a [D],
}

#[allow(dead_code)]
impl<S, D> FmtList<S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    pub fn new(separator: S) -> Self {
        Self {
            separator,
            elements: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn elements(&self) -> &[D] {
        &self.elements
    }

    pub fn separator(&self) -> &S {
        &self.separator
    }

    pub fn append(mut self, s: impl Into<D>) -> Self {
        self.elements.push(s.into());
        self
    }

    pub fn extend(mut self, s: impl IntoIterator<Item = impl Into<D>>) -> Self {
        self.elements.extend(s.into_iter().map(|s| s.into()));
        self
    }

    pub fn prepend(mut self, s: impl Into<D>) -> Self {
        self.elements.insert(0, s.into());
        self
    }

    pub fn pop(&mut self) -> Option<D> {
        self.elements.pop()
    }

    pub fn pop_front(&mut self) -> Option<D> {
        Some(self.elements.remove(0))
    }

    pub fn first(&self) -> Option<&D> {
        self.elements.first()
    }

    pub fn last(&self) -> Option<&D> {
        self.elements.last()
    }

    pub fn except_last(&self) -> FmtListSlice<'_, S, D> {
        let last = self.elements.len() - 1;
        FmtListSlice::new(&self.separator, self.elements().get(..last).unwrap_or(&[]))
    }

    pub fn last_slice(&self) -> FmtListSlice<'_, S, D> {
        let last = self.elements.len() - 1;
        if last == 0 {
            return FmtListSlice::new(&self.separator, &[]);
        }
        FmtListSlice::new(
            &self.separator,
            self.elements().get(last..=last).unwrap_or(&[]),
        )
    }

    pub fn as_slice(&self) -> FmtListSlice<'_, S, D> {
        FmtListSlice::new(&self.separator, self.elements.as_slice())
    }
}

#[allow(dead_code)]
impl<'a, S, D> FmtListSlice<'a, S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    fn new(separator: &'a S, elements: &'a [D]) -> Self {
        Self {
            separator,
            elements,
        }
    }

    pub fn first(&'a self) -> Option<&'a D> {
        self.elements.first()
    }

    pub fn last(&'a self) -> Option<&'a D> {
        self.elements.last()
    }

    pub fn separator(&'a self) -> &'a S {
        self.separator
    }

    pub fn elements(&'a self) -> &'a [D] {
        self.elements
    }
}

impl<S, D> std::fmt::Display for FmtList<S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_slice())
    }
}

impl<'a, S, D> std::fmt::Display for FmtListSlice<'a, S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
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

impl<S, D, IntoD, Separator, I> From<(Separator, I)> for FmtList<S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
    IntoD: Into<D>,
    Separator: Into<S>,
    I: IntoIterator<Item = IntoD>,
{
    fn from((separator, v): (Separator, I)) -> Self {
        Self {
            separator: separator.into(),
            elements: v.into_iter().map(|i| i.into()).collect(),
        }
    }
}

impl<'a, S, D> From<(&'a S, &'a [D])> for FmtListSlice<'a, S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    fn from((separator, elements): (&'a S, &'a [D])) -> Self {
        Self {
            separator,
            elements,
        }
    }
}

impl<S, D> IntoIterator for FmtList<S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    type Item = D;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, S, D> IntoIterator for FmtListSlice<'a, S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
{
    type Item = &'a D;
    type IntoIter = std::slice::Iter<'a, D>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<S, D, I> std::iter::Extend<I> for FmtList<S, D>
where
    S: std::fmt::Display,
    D: std::fmt::Display,
    I: Into<D>,
{
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        let iter = iter.into_iter().map(|s| s.into());
        self.elements.extend(iter);
    }
}
