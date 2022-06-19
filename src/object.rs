use crate::{string::LoxString, vm::Vm};
use core::ops::Add;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Str(LoxString),
}

impl Add<&Object> for &mut Object {
    type Output = Result<Self, ()>;

    fn add(mut self, rhs: &Object) -> Self::Output {
        match (&mut self, rhs) {
            (Object::Str(a), Object::Str(b)) => {
                // Discard the result because a's buffer is reused
                let _ = a.add(b);
                Ok(self)
            }
            _ => Err(()),
        }
    }
}

impl Object {
    // This will be useful later when we want to run something whenever we create a new string
    pub fn new_string(vm: &mut Vm, string: &str) -> Self {
        let lox_string = LoxString::from(string.to_string());
        vm.intern_string(&lox_string);
        Self::Str(lox_string)
    }
}
