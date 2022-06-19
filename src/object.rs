use core::ops::Add;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoxString {
    string: String,
}

impl From<String> for LoxString {
    fn from(string: String) -> Self {
        Self { string }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Str(LoxString),
}

impl Add<&Object> for &mut Object {
    type Output = Result<Self, ()>;

    fn add(mut self, rhs: &Object) -> Self::Output {
        match (&mut self, rhs) {
            (Object::Str(a), Object::Str(b)) => {
                a.string.push_str(b.string.as_str());
                Ok(self)
            }
            _ => Err(()),
        }
    }
}

impl Object {
    // This will be useful later when we want to run something whenever we create a new string
    pub fn new_string(string: &str) -> Self {
        Self::Str(LoxString::from(string.to_string()))
    }
}
