pub(self) trait Check<T> {
    fn check(self) -> Option<T>
    where
        Self: Sized;
}

impl Check<()> for bool {
    fn check(self) -> Option<()> {
        if self {
            Some(())
        } else {
            None
        }
    }
}

pub trait Goodbye<T>
where
    Self: Sized,
{
    /// Unwraps the contained value.<br>
    /// If unwrapping fails, prints `msg` and exits with code 1.
    #[momo::momo]
    fn bye(self, msg: impl AsRef<str>) -> T {
        if let Some(t) = self.check() {
            return t;
        }
        eprintln!("{}", msg);
        std::process::exit(1)
    }

    /// Unwraps the contained value.<br>
    /// If unwrapping fails, prints the result of `msg()` and exits with code 1.
    fn byefmt<S: AsRef<str>>(self, msg: impl Fn() -> S) -> T {
        if let Some(t) = self.check() {
            return t;
        }
        eprintln!("{}", msg().as_ref());
        std::process::exit(1)
    }

    /// Unwraps the contained value.<br>
    /// If unwrapping fails, calls `and` and exits with code 1.
    fn bye_and(self, and: impl FnOnce()) -> T {
        if let Some(t) = self.check() {
            return t;
        }
        and();
        std::process::exit(1);
    }

    fn check(self) -> Option<T>;
}

impl<T> Goodbye<T> for Option<T> {
    fn check(self) -> Option<T> {
        self
    }
}

impl<T, E> Goodbye<T> for Result<T, E> {
    fn check(self) -> Option<T> {
        self.ok()
    }
}

impl Goodbye<bool> for bool {
    fn check(self) -> Option<bool> {
        if self {
            Some(self)
        } else {
            None
        }
    }
}

pub trait OptionExt<T> {
    #[allow(clippy::wrong_self_convention)] // `is_some_and` takes `self` by value
    fn is_some_and_oneof<U>(self, of: impl AsRef<[U]>) -> bool
    where
        for<'a> &'a U: PartialEq<T>;

    fn drop_and<U>(self, and: U) -> Option<U>;
}

impl<T> OptionExt<T> for Option<T> {
    fn is_some_and_oneof<U>(self, of: impl AsRef<[U]>) -> bool
    where
        for<'a> &'a U: PartialEq<T>,
    {
        self.is_some_and(|s| of.as_ref().iter().any(|o| o == s))
    }

    /// Returns `Some(T)` if `self` is `None`, otherwise returns `None`.
    fn drop_and<U>(self, and: U) -> Option<U> {
        if self.is_some() {
            None
        } else {
            Some(and)
        }
    }
}

pub trait BoolExt: Check<()>
where
    Self: Sized,
{
    fn and<T>(self, and: T) -> Option<T> {
        if self.check().is_some() {
            Some(and)
        } else {
            None
        }
    }
    fn and_or<T>(self, and: T, or: T) -> T {
        if self.check().is_some() {
            and
        } else {
            or
        }
    }
    fn and_ok_or<T, E>(self, ok: T, or: E) -> Result<T, E> {
        if self.check().is_some() {
            Ok(ok)
        } else {
            Err(or)
        }
    }
}

impl BoolExt for bool {}