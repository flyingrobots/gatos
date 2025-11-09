use gatos_ledger_core::{ObjectStore, Hash};
use git2::{Repository}; // Removed Oid as it's not directly used
use std::vec::Vec;
use hex;
use blake3;

pub struct GitStore {
    repo: Repository,
}

impl GitStore {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}

impl ObjectStore for GitStore {
    fn put_object(&mut self, id: Hash, data: &[u8]) {
        let calculated_hash = blake3::hash(data);
        assert_eq!(calculated_hash.as_bytes(), &id, "Blake3 hash mismatch for data in GitStore::put_object"); // Compare &[] with &[]

        let git_oid = self.repo.odb().unwrap().write(git2::ObjectType::Blob, data).unwrap();

        let ref_name = format!("refs/gatos/blake3-map/{}", hex::encode(id));
        self.repo.reference(&ref_name, git_oid, true, "gatos: map blake3 hash to git oid").unwrap();
    }

    fn get_object(&self, id: &Hash) -> Option<Vec<u8>> {
        let ref_name = format!("refs/gatos/blake3-map/{}", hex::encode(id));

        let reference = self.repo.find_reference(&ref_name).ok()?;
        let git_oid = reference.target()?; // Directly get the Oid from a direct reference

        self.repo.find_blob(git_oid).ok().map(|b| b.content().to_vec())
    }
}