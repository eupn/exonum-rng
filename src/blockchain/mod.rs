pub mod schema;
pub mod errors;
pub mod transactions;

use exonum::crypto::{hash, Hash};

pub trait ToHash {
    fn to_hash(&self) -> Hash;
}

impl<T> ToHash for T where T: AsRef<str>, {
    fn to_hash(&self) -> Hash {
        hash(self.as_ref().as_bytes())
    }
}