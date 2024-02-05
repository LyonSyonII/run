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
}
