use std::cmp::{PartialEq, PartialOrd};
use std::fmt::Display;

use crate::object::Object;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Nil,
    Number(f64),
    Obj(Box<Object>),
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

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Obj(l0), Self::Obj(r0)) => **l0 == **r0,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Value {
    fn is_falsey(&self) -> bool {
        match *self {
            Value::Bool(value) => !value,
            Value::Nil => true,
            Value::Number(_) => false,
            Value::Obj(_) => false,
        }
    }
}

pub type ValueArray = Vec<Value>;

pub fn print_value(value: &Value) {
    print!("{}", *value)
}
