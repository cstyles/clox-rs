use crate::string::LoxString;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Str(LoxString),
}
