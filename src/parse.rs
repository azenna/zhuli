use crate::lex::*;

#[derive(Debug, Clone)]
pub enum StackError {
    StackEmpty,
    Val(ValueError),
}

#[derive(Debug, Default)]
pub struct Stack {
    pub fn_stack: Vec<Primitive>,
    pub val_stack: Vec<Value>,
}

impl Stack {
    pub fn dyadic(&mut self, f: fn(Value, Value) -> Result<Value, ValueError>) -> Result<(), StackError>{

        let a = self.value_pop()?;
        let b = self.value_pop()?;

        let new = f(a, b).map_err(StackError::Val)?;
        self.val_stack.push(new);
        Ok(())
    }
    pub fn add(&mut self) -> Result<(), StackError> {
        self.dyadic(Value::add)
    }
    pub fn sub(&mut self) -> Result<(), StackError> {
        self.dyadic(Value::sub)
    }
    pub fn lt(&mut self) -> Result<(), StackError> {
        self.dyadic(Value::lt)
    }
    pub fn gt(&mut self) -> Result<(), StackError> {
        self.dyadic(Value::gt)
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
    pub fn drop(&mut self) -> Result<(), StackError>{
        let _ = self.value_pop()?;
        Ok(())
    }
    pub fn select(&mut self) -> Result<(), StackError>{
        self.dyadic(Value::select)
    }
    pub fn exec(&mut self, f: Primitive) -> Result<(), StackError> {
        use Primitive::*;
        match f {
            Add => self.add()?,
            Sub => self.sub()?,
            Flip => self.flip()?,
            Duplicate => self.duplicate()?,
            GT => self.gt()?,
            LT => self.lt()?,
            Drop => self.drop()?,
            Select => self.select()?,
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
    OutOfBounds(Value),
}

#[derive(Debug, Clone)]
pub enum Value {
    Atom(Literal),
    Array(Vec<Value>),
}

use Value::Atom;
use Value::Array;

impl Value {
    pub fn dyadic_lits(self, other: Self, f: fn(Literal, Literal) -> Result<Literal, LiteralError>) -> Result<Self, ValueError> {
        match (self, other) {
            (Value::Atom(a), Value::Atom(b)) => Ok(Value::Atom(f(a, b).map_err(ValueError::Lit)?)),
            (Value::Array(arr), Value::Array(brr)) => {
                if arr.len() == brr.len(){
                    let mut new: Vec<Value> = vec![];
                    for (a, b) in arr.into_iter().zip(brr){
                        new.push(a.dyadic_lits(b, f)?);
                    }
                    Ok(Value::Array(new))

                } else {
                    Err(ValueError::ShapeMismatch)
                }
            }
            (a, Value::Array(arr)) => {
                let mut new: Vec<Value> = vec![];
                for val in arr{
                    new.push(a.clone().dyadic_lits(val, f)?);
                }
                Ok(Value::Array(new))
            }
            (Value::Array(arr), a) => {
                let mut new: Vec<Value> = vec![];
                for val in arr{
                    new.push(val.dyadic_lits(a.clone(), f)?);
                }
                Ok(Value::Array(new))
            }
        }
    }
    pub fn add(self, other: Self) -> Result<Self, ValueError> {
        self.dyadic_lits(other, Literal::add)
    }
    pub fn sub(self, other: Self) -> Result<Self, ValueError> {
        self.dyadic_lits(other, Literal::sub)
    }
    pub fn lt(self, other: Self) -> Result<Self, ValueError> {
        self.dyadic_lits(other, Literal::lt)
    }
    pub fn gt(self, other: Self) -> Result<Self, ValueError> {
        self.dyadic_lits(other, Literal::gt)
    }
    pub fn select(self, other: Self) -> Result<Self, ValueError> {
        match (self, other) {
            (Atom(a), Atom(b)) => {
                let num = a.as_number().map_err(ValueError::Lit)?;
                if num == 0f32 {
                    Ok(Atom(b))
                } else {
                    Err(ValueError::OutOfBounds(Atom(Literal::Number(num))))
                }
            }
            (Atom(a), Array(arr)) => {
                let i = a.as_number_whole().map_err(ValueError::Lit)?;
                if i < arr.len(){
                    Ok(arr[i].clone())
                } else {
                    Err(ValueError::OutOfBounds(Atom(Literal::Number(i as f32))))
                }
            }
            (Array(arr), val) => {
                let mut new: Vec<Value> = vec![];
                for v in arr{
                    new.push(v.select(val.clone())?);
                }
                Ok(Array(new))
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
