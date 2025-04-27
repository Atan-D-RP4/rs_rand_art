use logos::{Lexer, Logos};

#[derive(Clone, Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore whitespace between tokens
pub enum TokenKind {
    #[regex("#.*", logos::skip)]
    Comment,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r"[0-9]+")]
    Number,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    // Match '|', or '||' or '|||' or n pipes
    #[regex(r"\|{1,}")]
    Pipes,

    #[token(":")]
    Colon,

    #[token(";")]
    End,

    #[token(",")]
    Comma,

    #[token("::=")]
    ColonColonEqual,

    #[token("EOF")]
    EOF,
}

impl Default for TokenKind {
    fn default() -> Self {
        TokenKind::EOF
    }
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
        while let Some(token) = lexer.next() {
            tokens.push(lexer.slice());
        }
        println!("Tokens: {:?}", tokens);
        assert!(tokens.is_empty(), "No tokens were parsed");
    }
}

// Neovim command to send results of :!cargo clippy % to quickfix
// :cgetexpr system('cargo clippy %') | copen
