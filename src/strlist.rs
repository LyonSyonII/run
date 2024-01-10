use colored::{Color, Colorize, Style, Styles};

pub type Str<'a> = beef::lean::Cow<'a, str>;

#[derive(Debug, Clone)]
pub struct StrList<'a> {
    elements: Vec<Str<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub struct StrListSlice<'a> {
    elements: &'a [Str<'a>],
    color: Color,
    bold: bool,
}

impl<'a> StrList<'a> {
    pub fn new(separator: impl Into<Str<'a>>) -> Self {
        Self {
            elements: vec![separator.into()],
        }
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

impl<'a> StrListSlice<'a> {
    fn new(elements: &'a [Str<'a>]) -> Self {
        Self {
            elements,
            color: Color::White,
            bold: false,
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
        let mut iter = self.elements().iter().map(|s| {
            let mut s = s.color(self.color);
            if self.bold {
                s = s.bold()
            }
            s
        });
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
        // TODO: drain(1..) is a hack to remove the separator, can probably be done with unsafe and pointers
        self.elements.drain(1..).collect::<Vec<_>>().into_iter()
        /*
        let separator_ptr = self.elements.as_mut_ptr();
        let elements_ptr = unsafe { separator_ptr.add(1) };

        // SAFETY: If the original Vec only has one element (separator), len is 1, so the new iterator is empty
        let elements = if self.elements.len() <= 1 {
            Vec::new()
        }else {
            let length = self.elements.len() - 1;
            let capacity = self.elements.capacity() - 1;
            unsafe { Vec::from_raw_parts(elements_ptr, length, capacity) }
        };

        std::mem::forget(self);

        // SAFETY: `elements` always has at least one element, the separator, so it's safe to deallocate it
        unsafe { std::alloc::dealloc(separator_ptr as *mut u8, std::alloc::Layout::new::<Str>()) };
        elements.into_iter() */
    }
}

impl<'a> IntoIterator for StrListSlice<'a> {
    type Item = &'a Str<'a>;
    type IntoIter = std::slice::Iter<'a, Str<'a>>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.get(1..).unwrap_or_default().iter()
    }
}
