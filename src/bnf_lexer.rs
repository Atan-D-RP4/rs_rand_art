use logos::{Lexer, Logos};

// Example BNF grammar:
// # Entry
// E | vec3(C, C, C)
//   ;
//
// # Terminal
// A | random
//   | x
//   | y
//   | t
//   | abs(x)
//   | abs(y)
//   | sqrt(add(mult(x, x), mult(y, y))) # Distance from (0, 0) to (x, y)
//   ;
//
// # Expressions
// C ||  A
//   ||| add(C, C)
//   ||| mult(C, C)
//   | sqrt(abs(C))
//   # ||| abs(C)
//   #||| sin(C)
//   ;
//

#[derive(Clone, Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+|#.*")]
#[derive(Default)]
pub enum TokenKind {
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"[0-9]+")]
    Number,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    // Match '|', or '||' or '|||' or n pipes
    #[regex(r"\|{1,}")]
    Pipes,

    #[token(":")]
    Colon,

    #[token(";")]
    End,

    #[token(",")]
    Comma,

    #[token("EOF")]
    #[default]
    EOF,
}

impl TokenKind {
    pub fn new(input: &'static str) -> Lexer<'static, Self> {
        TokenKind::lexer(&input)
    }
}

#[cfg(test)]
mod test {
    use super::TokenKind;
    use logos::Logos;
    use std::fs;

    #[test]
    fn test_lex() {
        let input = fs::read_to_string("./grammar.bnf").unwrap_or_else(|err| {
            eprintln!("Failed to read file: {err}");
            String::new()
        });
        let mut lexer = TokenKind::lexer(&input);
        let mut tokens = Vec::new();
        while let Some(Ok(token)) = lexer.next() {
            tokens.push((token, lexer.slice()));
        }
        for (token, slice) in &tokens {
            println!("{token:?} :: {slice}");
        }
        assert!(!tokens.is_empty(), "No tokens were parsed");
    }
}

// Neovim command to send results of :!cargo clippy % to quickfix
// :cgetexpr system('cargo clippy %') | copen
