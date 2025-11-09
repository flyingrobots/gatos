# GATOS Ledger Git Backend

This crate provides a `std`-dependent storage backend for the GATOS ledger that uses `libgit2`. It implements the `ObjectStore` trait from `gatos-ledger-core`, acting as an adapter to connect the core ledger logic to a real Git repository on a filesystem.

## Usage

```rust
use git2::Repository;
use gatos_ledger_git::GitStore;
use gatos_ledger_core::{ObjectStore, Hash};
use blake3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Open an existing repository
    let repo = Repository::open("/path/to/repo")?;
    let mut store = GitStore::new(repo);

    // Put some bytes under their blake3 hash
    let data = b"hello";
    let id: Hash = blake3::hash(data).into();
    store.put_object(&id, data)?;

    // Retrieve them later
    if let Some(bytes) = store.get_object(&id)? {
        assert_eq!(bytes, data);
    }
    Ok(())
}
```

For more details on the overall architecture, see the main [GATOS Technical Specification](../../docs/TECH-SPEC.md).
