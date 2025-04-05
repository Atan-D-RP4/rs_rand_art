use logos::Logos;

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

    #[token("EOF")]
    EOF,
}

#[cfg(test)]
mod test {
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
            grammar_lang::Token::Identifier,
            grammar_lang::Token::ColonColonEqual,
            grammar_lang::Token::Identifier,
            grammar_lang::Token::Pipe,
            grammar_lang::Token::Identifier,
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
