#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Bash;

impl super::Language for Bash {
    fn as_str(&self) -> &'static str {
        "bash"
    }

    fn binary(&self) -> &'static str {
        "bash"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["bash"]
    }

    fn command_call<'a, D>(&'a self, args: impl IntoIterator<Item = &'a D> + Clone) -> String
    where
        D: std::fmt::Display + ?Sized + 'a,
    {
        let args = crate::fmt::strlist::FmtIter::new(" ", args);
        format!("run {}", args)
    }
}
