#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Python;

impl super::Language for Python {
    fn as_str(&self) -> &'static str {
        "py"
    }

    fn binary(&self) ->  &'static str {
        "python"
    }
    
    fn nix_packages(&self) ->  &'static[&'static str] {
        &["python3Minimal"]
    }
}
