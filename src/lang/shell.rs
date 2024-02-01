#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Shell;

impl super::Language for Shell {
    fn as_str(&self) -> &'static str {
        "sh"
    }

    fn binary(&self) -> &'static str {
        "sh"
    }

    fn nix_packages(&self) ->  &'static[&'static str] {
        &[]
    }
}
