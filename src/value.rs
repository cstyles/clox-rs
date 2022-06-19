use std::cmp::{PartialEq, PartialOrd};
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::ops::Not for &Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        Value::Bool(self.is_falsey())
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Number(left), Value::Number(right)) => left.partial_cmp(right),
            _ => None,
        }
    }
}

impl Value {
    pub fn as_bool(&self) -> bool {
        match *self {
            Self::Bool(value) => value,
            _ => panic!("{self} wasn't a bool!"),
        }
    }

    pub fn as_number(&self) -> f64 {
        match *self {
            Self::Number(value) => value,
            _ => panic!("{self} wasn't a number!"),
        }
    }

    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(..) => true,
            _ => false,
        }
    }

    pub fn is_nil(&self) -> bool {
        match self {
            Self::Nil => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self {
            Self::Number(..) => true,
            _ => false,
        }
    }

    fn is_falsey(&self) -> bool {
        match *self {
            Value::Bool(value) => !value,
            Value::Nil => true,
            Value::Number(_) => false,
        }
    }
}

pub type ValueArray = Vec<Value>;

pub fn print_value(value: &Value) {
    print!("{}", *value)
}
