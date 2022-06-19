use core::ops::Add;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LoxString {
    string: String,
    hash: u64,
}

impl Add<&LoxString> for &mut LoxString {
    type Output = Self;

    fn add(self, rhs: &LoxString) -> Self::Output {
        self.string.push_str(rhs.string.as_str());
        self.rehash();
        self
    }
}

impl From<String> for LoxString {
    fn from(string: String) -> Self {
        let hash = Self::hash(&string);
        Self { string, hash }
    }
}

impl LoxString {
    fn hash(string: &str) -> u64 {
        let mut hasher = FnvHasher::default();
        string.hash(&mut hasher);
        hasher.finish()
    }

    fn rehash(&mut self) {
        self.hash = Self::hash(&self.string);
    }
}
