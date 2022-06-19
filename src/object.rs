use core::ops::Add;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Object {
    Str(String),
}

impl Add<&Object> for &mut Object {
    type Output = Result<Self, ()>;

    fn add(mut self, rhs: &Object) -> Self::Output {
        match (&mut self, rhs) {
            (Object::Str(a), Object::Str(b)) => {
                a.push_str(b);
                Ok(self)
            }
            _ => Err(()),
        }
    }
}

impl Object {
    // This will be useful later when we want to run something whenever we create a new string
    pub fn new_string(string: &str) -> Self {
        Self::Str(string.to_string())
    }
}
