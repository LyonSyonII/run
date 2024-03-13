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
        &["dart"]
    }

    fn execute(&self, input: &str, args: impl AsRef<[String]>) -> Result<(), crate::fmt::Str<'_>> {
        let input = format!("void main() {{\n{}\n}}", input);
        super::execute_interpreted(self.as_str(), self.program()?, &input, args)
    }
}
