use logos::Logos;
use crate::grammar::*;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
pub enum Token {
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

    #[token("|")]
    Pipe,

    #[token(":")]
    Colon,

    #[token(";")]
    End,

    #[token(",")]
    Comma,

    #[token("::=")]
    ColonColonEqual,
}

impl Grammar {
    pub fn from_bnf(bnf_source: String) -> Result<Grammar, String> {
        let lexer = Token::lexer(&bnf_source);
        let mut grammar = Grammar::new();

        match grammar.parse(lexer) {
            Ok(()) => Ok(grammar),
            Err(e) => Err(e),
        }
    }

    fn parse(&mut self, lexer: logos::Lexer<Token>) -> Result<(), String> {
        let mut lexer = lexer.peekable();
        while let Some(Ok(token)) = lexer.next() {
            match token {
                _ => todo!()
            }
        }
        Ok(())
    }

}


#[cfg(test)]
mod test {
    use super::Token;
    use logos::Logos;
    #[test]
    fn test_lex() {
        let input = r#"
a ::=
    ||| b
    | c
    ;
        "#;
        let expected = vec![
            Token::Identifier,
            Token::ColonColonEqual,
            Token::Identifier,
            Token::Pipe,
            Token::Identifier,
        ];
        let lexer = super::Token::lexer(input);
        let mut tokens = Vec::new();
        lexer.for_each(|x| {
            let x = x.unwrap();
            tokens.push(x)
        });
        assert_eq!(tokens, expected);
    }
}
