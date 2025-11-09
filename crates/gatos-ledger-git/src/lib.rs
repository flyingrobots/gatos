use gatos_ledger_core::{ObjectStore, Hash};
use git2::{Repository, Oid};
use std::vec::Vec;

pub struct GitStore {
    repo: Repository,
}

impl GitStore {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}

impl ObjectStore for GitStore {
    fn put_object(&mut self, _id: Hash, data: &[u8]) {
        self.repo.odb().unwrap().write(git2::ObjectType::Blob, data).unwrap();
    }

    fn get_object(&self, id: &Hash) -> Option<Vec<u8>> {
        self.repo.find_blob(Oid::from_bytes(id).ok()?).ok().map(|b| b.content().to_vec())
    }
}