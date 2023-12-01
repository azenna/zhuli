#[derive(Debug, Clone)]
pub enum Primitive {
    Add,
    Sub,
    Flip,
    Duplicate,
}

impl Into<Token> for Primitive {
    fn into(self) -> Token {
        Token::Pri(self)
    }
}

#[derive(Debug, Clone)]
pub enum LiteralError {
    Unit,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f32),
}

impl Literal {
    pub fn add(self, other: Self) -> Result<Self, LiteralError> {
        match (self, other) {
            (Literal::Number(a), Literal::Number(b)) => Ok(Literal::Number(a + b)),
            _ => Err(LiteralError::Unit),
        }
    }
}

impl Into<Token> for Literal {
    fn into(self) -> Token {
        Token::Lit(self)
    }
}

impl Literal {
    pub fn as_number(self) -> Option<f32> {
        match self {
            Literal::Number(f) => Some(f),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Pri(Primitive),
    Lit(Literal),
    LBracket,
    RBracket,
}

#[derive(Debug)]
pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Lexer {
            src,
            pos: 0,
            tokens: vec![],
        }
    }

    pub fn peek_char(&self) -> Option<char> {
        self.src.chars().nth(self.pos + 1)
    }

    pub fn next_char(&mut self) -> Option<char> {
        let c = self.src.chars().nth(self.pos)?;
        self.pos += 1;
        Some(c)
    }

    pub fn add_token(&mut self, tok: impl Into<Token>) {
        self.tokens.push(tok.into())
    }

    pub fn run(mut self) -> Vec<Token> {
        use Literal::*;
        use Primitive::*;
        use Token::*;

        loop {
            let Some(c) = self.next_char() else {
                break;
            };
            match c {
                '+' => self.add_token(Add),
                '-' => self.add_token(Sub),
                ':' => self.add_token(Flip),
                '.' => self.add_token(Duplicate),
                '[' => self.add_token(LBracket),
                ']' => self.add_token(RBracket),
                c if c.is_digit(10) => self.add_token(Number(c.to_digit(10).unwrap() as f32)),
                _ => (),
            }
        }

        self.tokens
    }
}
