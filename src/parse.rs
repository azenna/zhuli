use crate::lex::*;

#[derive(Debug, Clone)]
pub enum StackError {
    TypeMismatch,
    StackEmpty,
    Val(ValueError),
}

#[derive(Debug, Default)]
pub struct Stack {
    pub fn_stack: Vec<Primitive>,
    pub val_stack: Vec<Value>,
}

impl Stack {
    pub fn add(&mut self) -> Result<(), StackError> {
        let a = self.value_pop()?;
        let b = self.value_pop()?;

        let new = a.add(b).map_err(StackError::Val)?;

        self.val_stack.push(new);

        Ok(())
    }
    pub fn sub(&mut self) -> Result<(), StackError> {
        self.dyadic_atom_num(|a, b| b - a)
    }
    pub fn value_last(&self) -> Result<&Value, StackError> {
        self.val_stack.last().ok_or(StackError::StackEmpty)
    }
    pub fn value_pop(&mut self) -> Result<Value, StackError> {
        self.val_stack.pop().ok_or(StackError::StackEmpty)
    }
    pub fn flip(&mut self) -> Result<(), StackError> {
        let a = self.value_pop()?;
        let b = self.value_pop()?;

        self.val_stack.push(a);
        self.val_stack.push(b);

        Ok(())
    }
    pub fn duplicate(&mut self) -> Result<(), StackError> {
        let new = self.value_last()?.clone();
        self.val_stack.push(new);

        Ok(())
    }
    pub fn dyadic_atom_num(&mut self, f: fn(f32, f32) -> f32) -> Result<(), StackError> {
        let a = self
            .value_pop()?
            .as_atom()
            .ok_or(StackError::TypeMismatch)?
            .as_number()
            .ok_or(StackError::TypeMismatch)?;

        let b = self
            .value_pop()?
            .as_atom()
            .ok_or(StackError::TypeMismatch)?
            .as_number()
            .ok_or(StackError::TypeMismatch)?;

        self.val_stack.push(Value::Atom(Literal::Number(f(a, b))));

        Ok(())
    }
    pub fn exec(&mut self, f: Primitive) -> Result<(), StackError> {
        use Primitive::*;
        match f {
            Add => self.add()?,
            Sub => self.sub()?,
            Flip => self.flip()?,
            Duplicate => self.duplicate()?,
        };

        Ok(())
    }

    pub fn run(mut self) -> Result<Vec<Value>, StackError> {
        while let Some(f) = self.fn_stack.pop() {
            self.exec(f)?;
        }
        Ok(self.val_stack)
    }
}

#[derive(Debug, Clone)]
pub enum ValueError {
    Lit(LiteralError),
    ShapeMismatch,
}

#[derive(Debug, Clone)]
pub enum Value {
    Atom(Literal),
    Array(Vec<Value>),
}

impl Value {
    pub fn add(self, other: Self) -> Result<Self, ValueError> {
        match (self, other) {
            (Value::Atom(a), Value::Atom(b)) => Ok(Value::Atom(a.add(b).map_err(ValueError::Lit)?)),
            (Value::Atom(a), Value::Array(arr)) => {
                let mut new: Vec<Value> = vec![];
                for val in arr{
                    new.push(Value::Atom(a.clone()).add(val)?);
                }
                Ok(Value::Array(new))
            }
            (Value::Array(arr), Value::Atom(a)) => {
                let mut new: Vec<Value> = vec![];
                for val in arr{
                    new.push(Value::Atom(a.clone()).add(val)?);
                }
                Ok(Value::Array(new))
            }
            (Value::Array(arr), Value::Array(brr)) => {
                if arr.len() == brr.len(){
                    let mut new: Vec<Value> = vec![];
                    for (a, b) in arr.into_iter().zip(brr){
                        new.push(a.add(b)?);
                    }
                    Ok(Value::Array(new))

                } else {
                    Err(ValueError::ShapeMismatch)
                }
            }
        }
    }
}

impl Value {
    pub fn as_atom(self) -> Option<Literal> {
        match self {
            Value::Atom(lit) => Some(lit),
            _ => None,
        }
    }
}

pub enum ParseError {
    Unit,
}

pub struct Parse {
    toks: Vec<Token>,
    index: usize,
    stack: Stack,
    errors: Vec<ParseError>,
}

impl Parse {
    pub fn new(toks: Vec<Token>) -> Parse {
        Parse {
            toks,
            index: 0usize,
            stack: Stack::default(),
            errors: vec![],
        }
    }
    pub fn next_token(&mut self) -> Option<Token> {
        self.index += 1;
        self.toks.get(self.index).cloned()
    }

    pub fn parse(mut self) -> (Stack, Vec<ParseError>) {
        use Token::*;

        while let Some(tok) = self.toks.get(self.index) {
            match tok {
                LBracket => {
                    let val = self.parse_array();
                    self.stack.val_stack.insert(0, val);
                }
                RBracket => self.errors.push(ParseError::Unit),
                Lit(lit) => self.stack.val_stack.insert(0, Value::Atom(lit.clone())),
                Pri(pri) => self.stack.fn_stack.push(pri.clone()),
            }
            self.index += 1;
        }

        (self.stack, self.errors)
    }

    pub fn parse_array(&mut self) -> Value {
        let mut arr: Vec<Value> = vec![];
        loop {
            if let Some(tok) = self.next_token() {
                match tok {
                    Token::Lit(lit) => arr.push(Value::Atom(lit)),
                    Token::LBracket => arr.push(self.parse_array()),
                    Token::RBracket => break,
                    _ => self.errors.push(ParseError::Unit),
                }
            } else {
                break;
            }
        }
        Value::Array(arr)
    }
}
