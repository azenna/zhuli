#[derive(Debug, Clone)]
pub enum Primitive {
    Add,
    Sub,
    Flip,
    Duplicate,
    Drop,
    LT,
    GT,
    Select,
}

impl Into<Token> for Primitive {
    fn into(self) -> Token {
        Token::Pri(self)
    }
}

#[derive(Debug, Clone)]
pub enum LiteralError {
    Unit,
    NotNumber,
    ExpectedWhole,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f32),
}

impl Literal {
    pub fn as_number(self) -> Result<f32, LiteralError> {
        match self {
            Literal::Number(f) => Ok(f),
            _ => Err(LiteralError::NotNumber),
        }
    }
    pub fn as_number_whole(self) -> Result<usize, LiteralError> {
        self.as_number().and_then(|x| if x.fract() == 0f32 {Ok(x as usize)} else {Err(LiteralError::ExpectedWhole)})
    }
    pub fn dyadic_nums(self, other: Self, f: fn(f32, f32) -> f32) -> Result<Self, LiteralError>{
        match (self, other) {
            (Literal::Number(a), Literal::Number(b)) => Ok(Literal::Number(f(a, b))),
            _ => Err(LiteralError::Unit),
        }
    }
    pub fn add(self, other: Self) -> Result<Self, LiteralError> {
        self.dyadic_nums(other, |a, b| a + b)
    }
    pub fn sub(self, other: Self) -> Result<Self, LiteralError> {
        self.dyadic_nums(other, |a, b| b - a )
    }
    pub fn lt(self, other: Self) -> Result<Self, LiteralError> {
        self.dyadic_nums(other, |a, b| (b < a ) as u32 as f32)
    }
    pub fn gt(self, other: Self) -> Result<Self, LiteralError> {
        self.dyadic_nums(other, |a, b| (b > a) as u32 as f32)
    }
}

impl Into<Token> for Literal {
    fn into(self) -> Token {
        Token::Lit(self)
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
                '>' => self.add_token(GT),
                '<' => self.add_token(LT),
                ';' => self.add_token(Drop),
                '⊏' => self.add_token(Select),

                c if c.is_digit(10) => self.add_token(Number(c.to_digit(10).unwrap() as f32)),
                _ => (),
            }
        }

        self.tokens
    }
}
