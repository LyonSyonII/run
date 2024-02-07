#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Dart;

impl super::Language for Dart {
    fn as_str(&self) -> &'static str {
        "dart" 
    }

    fn binary(&self) -> &'static str {
        "dart"
    }

    fn nix_packages(&self) -> &'static [&'static str] {
        &[]
    }
}