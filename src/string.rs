use fnv::FnvHasher;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::vm::Vm;

#[derive(Debug, Clone, Eq)]
pub struct LoxString {
    string: Rc<String>,
    hash: u64,
}

impl PartialEq<Self> for LoxString {
    fn eq(&self, other: &Self) -> bool {
        // We intern strings so we know that if the pointers (`Rc<String>`) that the
        // `LoxString`s contain aren't equal, their data must also be different so we
        // don't need to bother checking every character.
        self.string.as_ptr() == other.string.as_ptr()
    }
}

impl From<Rc<String>> for LoxString {
    fn from(string: Rc<String>) -> Self {
        let mut hasher = FnvHasher::default();
        string.hash(&mut hasher);
        let hash = hasher.finish();

        Self { string, hash }
    }
}

impl Hash for LoxString {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl LoxString {
    // This will be useful later when we want to run something whenever we create a new string
    // TODO: impl ToString / Cow?
    pub fn copy_string(vm: &mut Vm, string: &str) -> Self {
        vm.intern_string(string.to_string())
    }

    // This will be useful later when we want to run something whenever we create a new string
    fn take_string(vm: &mut Vm, string: String) -> Self {
        vm.intern_string(string)
    }

    pub fn add(vm: &mut Vm, a: &LoxString, b: &LoxString) -> Self {
        let new_string = format!("{}{}", a.string, b.string);
        Self::take_string(vm, new_string)
    }
}
