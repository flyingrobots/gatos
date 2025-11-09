#![no_std]

extern crate alloc;
use alloc::vec::Vec;

use serde_with::serde_as;

pub type Hash = [u8; 32];

pub trait ObjectStore {
    fn put_object(&mut self, id: Hash, data: &[u8]);
    fn get_object(&self, id: &Hash) -> Option<Vec<u8>>;
}

#[serde_as]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Commit {
    pub parent: Option<Hash>,
    pub tree: Hash,
    #[serde_as(as = "[_; 64]")]
    pub signature: [u8; 64],
}

pub fn compute_commit_id(commit: &Commit) -> Hash {
    blake3::hash(&bincode::serialize(commit).unwrap()).into()
}
