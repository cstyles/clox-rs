use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

use crate::vm::Vm;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoxString {
    string: String,
    hash: u64,
}

impl From<String> for LoxString {
    fn from(string: String) -> Self {
        let mut hasher = FnvHasher::default();
        string.hash(&mut hasher);
        let hash = hasher.finish();

        Self { string, hash }
    }
}

impl LoxString {
    // This will be useful later when we want to run something whenever we create a new string
    // TODO: impl ToString / Cow?
    pub fn copy_string(vm: &mut Vm, string: &str) -> Self {
        let lox_string = LoxString::from(string.to_string());
        vm.intern_string(&lox_string);
        lox_string
    }

    // This will be useful later when we want to run something whenever we create a new string
    fn take_string(vm: &mut Vm, string: String) -> Self {
        let lox_string = LoxString::from(string);
        vm.intern_string(&lox_string);
        lox_string
    }

    pub fn add(vm: &mut Vm, a: &LoxString, b: &LoxString) -> LoxString {
        let new_string = format!("{}{}", a.string, b.string);
        Self::take_string(vm, new_string)
    }
}
