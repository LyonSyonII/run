#![cfg(test)]
use super::*;

mod parser {
    use crate::parsing::{functions::ParseFunctions, parser::Parser};

    use super::error::Error;

    #[test]
    fn consume_n() {
        fn consume_n<'i>(parser: &mut Parser<'i>, n: usize) -> Result<&'i str, Error> {
            parser.consume_n(n).map(|r| r.0).map_err(|e| e.0)
        }
        let parser = &mut super::Parser::new("123456789");
        
        assert_eq!(consume_n(parser, 5), Ok("12345"));
        assert_eq!(parser.pos(), 5);
        assert_eq!(parser.input(), "6789");

        assert_eq!(consume_n(parser, 1), Ok("6"));
        assert_eq!(parser.input(), "789");

        assert_eq!(consume_n(parser, 2), Ok("78"));
        assert_eq!(parser.input(), "9");

        assert!(consume_n(parser, 55).is_err());
        assert_eq!(consume_n(parser, 1), Ok("9"));
        assert_eq!(parser.input(), "");
    }

    #[test]
    fn peek_char_is() {
        let parser = &mut Parser::new("aA1!");
        assert!(parser.peek_char_is(|c| c == Some('a')).is_ok());
        assert!(parser
            .peek_char_is(|c| c.is_some_and(|c| c.is_alphabetic() && c.is_lowercase()))
            .is_ok());
        assert_eq!(parser.pos(), 0);
        assert!(parser.peek_char_is(|c| c == Some('A')).is_err());

        // let c = parser.checkpoint();
        // let (s, c) = c.consume_n(1);
        // c.consume_n(2);
    }
}
