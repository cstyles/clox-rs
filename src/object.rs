use std::fmt::Display;

use crate::string::LoxString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Str(LoxString),
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Str(lox_string) => write!(f, "{}", lox_string),
        }
    }
}

impl Object {
    pub fn as_string(&self) -> &LoxString {
        if let Object::Str(string) = self {
            string
        } else {
            panic!("Object wasn't a string.");
        }
    }
}
