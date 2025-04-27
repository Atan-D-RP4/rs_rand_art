// src/bnf_parser.rs

use std::collections::HashMap;

use logos::Lexer;

use crate::grammar::{Branch, Grammar, Rule};
use crate::node::{ArithmeticOp, FnNode, UnaryOp};

use crate::bnf_lexer::TokenKind;

pub struct BNFParser {
    pub lexer: Lexer<'static, TokenKind>,
    pub grammar: Grammar,
}

impl BNFParser {
    pub fn new(input: &'static str) -> Self {
        let lexer = TokenKind::new(input);
        let grammar = Grammar::new();
        BNFParser { lexer, grammar }
    }

    pub fn parse(&mut self) -> Result<Grammar, String> {
        todo!("Implement the BNF parser");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_bnf() {
        let input = r"
E | vec3(X, X, X)
  ;
        ";
        let mut parser = BNFParser::new(input);
        let result = parser.parse();

        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
