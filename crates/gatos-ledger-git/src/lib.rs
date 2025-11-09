pub use gatos_ledger_core::*; // Re-export core API surface for facade users
use git2::Repository;

pub struct GitStore {
    repo: Repository,
}

impl GitStore {
    #[must_use]
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}

impl ObjectStore for GitStore {
    fn put_object(&mut self, id: &Hash, data: &[u8]) -> Result<(), StoreError> {
        let calculated_hash = blake3::hash(data);
        if calculated_hash.as_bytes() != id {
            return Err(StoreError::Corruption);
        }

        let odb = self.repo.odb().map_err(|_| StoreError::Io)?;
        let git_oid = odb
            .write(git2::ObjectType::Blob, data)
            .map_err(|_| StoreError::Io)?;

        let ref_name = format!("refs/gatos/blake3-map/{}", hex::encode(id));
        self.repo
            .reference(
                &ref_name,
                git_oid,
                true,
                "gatos: map blake3 hash to git oid",
            )
            .map_err(|_| StoreError::Io)?;
        Ok(())
    }

    fn get_object(&self, id: &Hash) -> Result<Option<Vec<u8>>, StoreError> {
        let ref_name = format!("refs/gatos/blake3-map/{}", hex::encode(id));
        let reference = match self.repo.find_reference(&ref_name) {
            Ok(r) => r,
            Err(e) => {
                // If reference does not exist, treat as not found; other errors as IO
                if e.code() == git2::ErrorCode::NotFound {
                    return Ok(None);
                }
                return Err(StoreError::Io);
            }
        };
        let Some(git_oid) = reference.target() else {
            return Err(StoreError::Invariant);
        };
        let blob = self.repo.find_blob(git_oid).map_err(|_| StoreError::Io)?;
        Ok(Some(blob.content().to_vec()))
    }
}
