#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shell;

impl super::Language for Shell {
    fn as_str(&self) -> &'static str {
        "sh"
    }

    fn binary(&self) -> &'static str {
        "sh"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &[]
    }

    fn command_call<'a, D>(
        &'a self,
        command: &str,
        args: impl IntoIterator<Item = &'a D> + Clone,
    ) -> String
    where
        D: std::fmt::Display + ?Sized + 'a,
    {
        let args = crate::fmt::strlist::FmtIter::new(&" ", args);
        format!("run {} {}", command, args)
    }
}
