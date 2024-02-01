#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Javascript;

impl super::Language for Javascript {
    fn as_str(&self) -> &'static str {
        "js"
    }

    fn binary(&self) -> &'static str {
        "node"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &["nodejs"]
    }
}
