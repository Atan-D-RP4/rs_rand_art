use image::codecs::webp;
use logos::Logos;
use std::collections::HashMap;

// Importing your existing types
use crate::node::{self, ArithmeticOp, CompareOp, FnNode, UnaryOp};

// We'll assume these types are already defined in your project
use crate::grammar::{Branch, Grammar, Rule};

use crate::bnf_lexer::TokenKind;

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(TokenKind),
    ExpectedIdentifier,
    ExpectedColonColonEqual,
    ExpectedEnd,
    InvalidBranchWeight,
    InvalidRule,
    UnknownSymbol(String),
    UnknownFunction(String),
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    lexer: logos::Lexer<'a, TokenKind>,
    symbol_map: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = TokenKind::lexer(input);

        Parser {
            lexer,
            symbol_map: Vec::new(),
        }
    }

    pub fn collect_symbols(&mut self) -> Result<(), ParseError> {
        let mut lexer = self.lexer.clone();
        let mut ended = true;

        while let Some(Ok(token)) = lexer.next() {
            match token {
                TokenKind::Identifier => {
                    let symbol = lexer.slice();
                    if lexer.next() == Some(Ok(TokenKind::Pipes)) && ended {
                        self.symbol_map.push(symbol.to_string());
                        ended = false;
                    }
                }

                TokenKind::End => {
                    ended = true;
                    continue;
                }

                TokenKind::EOF => {
                    break;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn parse(&mut self) -> Result<Grammar, ParseError> {
        let mut grammar = Grammar::new();
        self.collect_symbols()?;
        println!("Symbols: {:?}", self.symbol_map);

        while let Some(Ok(token)) = self.lexer.next() {
            match token {
                TokenKind::Identifier => {
                    let symbol = self.lexer.slice();
                    if self.symbol_map.contains(&symbol.to_string()) {
                        self.parse_rule(symbol, &mut grammar)?;
                    } else {
                        return Err(ParseError::UnknownSymbol(symbol.to_string()));
                    }
                }
                TokenKind::End => continue,
                TokenKind::EOF => break,
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        Ok(grammar)
    }

    pub fn parse_rule(&mut self, symbol: &str, grammar: &mut Grammar) -> Result<(), ParseError> {
        let mut branches: Vec<Branch> = Vec::new();

        while let Some(Ok(token)) = self.lexer.next() {
            match token {
                TokenKind::Pipes => {
                    let weight = self.lexer.slice().len();
                    let branch = self.parse_branch(weight)?;
                    branches.push(branch);
                }
                TokenKind::End | TokenKind::EOF => break,
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }
        if branches.is_empty() {
            return Err(ParseError::InvalidRule);
        }
        grammar
            .add_rule(branches, symbol.to_string())
            .map_err(|_| ParseError::InvalidRule)?;
        Ok(())
    }

    pub fn parse_branch(&mut self, weight: usize) -> Result<Branch, ParseError> {
        if let Some(Ok(token)) = self.lexer.next() {
            match token {
                TokenKind::Identifier => {
                    let ident = self.lexer.slice();
                    return Ok(Branch::new(self.parse_ident(ident)?, weight));
                }
                TokenKind::End | TokenKind::EOF => {}
                _ => return Err(ParseError::UnexpectedToken(token)),
            }
        }

        Err(ParseError::InvalidBranchWeight)
    }

    pub fn parse_ident(&mut self, ident: &str) -> Result<FnNode, ParseError> {
        if self.symbol_map.contains(&ident.to_string()) {
            return Ok(FnNode::Rule(
                match self.symbol_map.iter().position(|s| s == ident) {
                    Some(idx) => idx,
                    None => return Err(ParseError::UnknownSymbol(ident.to_string())),
                },
            ));
        }
        match ident {
            "X" | "x" => Ok(FnNode::X),
            "Y" | "y" => Ok(FnNode::Y),
            "Z" | "z" | "T" | "t" => Ok(FnNode::T),
            "random" => Ok(FnNode::Random),
            "vec3" => {
                let mut nodes = Vec::new();
                while let Some(Ok(token)) = self.lexer.next() {
                    match token {
                        TokenKind::Identifier => {
                            let arg = self.lexer.slice().to_string();
                            nodes.push(self.parse_ident(&arg)?);
                        }
                        TokenKind::Comma | TokenKind::LParen => {}
                        TokenKind::RParen => break,
                        _ => return Err(ParseError::UnexpectedToken(token)),
                    }
                }
                if nodes.len() != 3 {
                    // return Err("vec3 requires exactly 3 arguments".to_string());
                    return Err(ParseError::InvalidRule);
                }
                Ok(FnNode::triple(
                    nodes[0].clone(),
                    nodes[1].clone(),
                    nodes[2].clone(),
                ))
            }
            "add" | "mult" | "sub" | "div" | "mod" => {
                let mut nodes = Vec::with_capacity(3);
                while let Some(Ok(token)) = self.lexer.next() {
                    match token {
                        TokenKind::Identifier => {
                            let arg = self.lexer.slice().to_string();
                            nodes.push(self.parse_ident(&arg)?);
                        }
                        TokenKind::Comma | TokenKind::LParen => {}
                        TokenKind::RParen => break,
                        _ => return Err(ParseError::UnexpectedToken(token)),
                    }
                }

                if nodes.len() != 2 {
                    return Err(ParseError::InvalidRule);
                }

                let op = match ident {
                    "add" => ArithmeticOp::Add,
                    "sub" => ArithmeticOp::Sub,
                    "mult" | "mul" => ArithmeticOp::Mul,
                    "div" => ArithmeticOp::Div,
                    "mod" => ArithmeticOp::Mod,
                    _ => unreachable!(),
                };

                Ok(FnNode::arithmetic(nodes[0].clone(), op, nodes[1].clone()))
            }

            "sqrt" | "abs" | "sin" | "tan" | "cos" => {
                let mut nodes = Vec::new();
                while let Some(Ok(token)) = self.lexer.next() {
                    match token {
                        TokenKind::Identifier => {
                            let arg = self.lexer.slice().to_string();
                            nodes.push(self.parse_ident(&arg)?);
                        }
                        TokenKind::Comma | TokenKind::LParen => {}
                        TokenKind::RParen => break,
                        _ => return Err(ParseError::UnexpectedToken(token)),
                    }
                }
                if nodes.len() != 1 {
                    println!("Invalid number of arguments: {nodes:?}");
                    return Err(ParseError::InvalidRule);
                }

                let op = match ident {
                    "sqrt" => UnaryOp::Sqrt,
                    "abs" => UnaryOp::Abs,
                    "sin" => UnaryOp::Sin,
                    "tan" => UnaryOp::Tan,
                    "cos" => UnaryOp::Cos,
                    _ => unreachable!(),
                };
                Ok(FnNode::unary(op, nodes[0].clone()))
            }
            // Handle rule references
            _ => {
                // Assume it's a reference to another rule
                // This part would need more context to properly implement
                Err(ParseError::UnknownSymbol(ident.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse_bnf() {
        let input = r"
# Entry
E | vec3(C, C, C)
  ;

# Terminal
A | random
  | x
  | y
  | t
  | abs(x)
  | abs(y)
  | sqrt(add(mult(x, x), mult(y, y))) # Distance from (0, 0) to (x, y)
  ;

# Expressions
C ||  A
  ||| add(C, C)
  ||| mult(C, C)
  | sqrt(abs(C))
  # ||| abs(C)
  #||| sin(C)
  ;
        ";
        let mut parser = Parser::new(input);
        let result = parser.parse();
        assert!(result.is_ok(), "Parse should be successful");
        let _ = result.map(|grammar| {
            let node = grammar.gen_from_rule(0, 10);
            assert!(node.is_some(), "Node should be generated");
        });
    }
}
