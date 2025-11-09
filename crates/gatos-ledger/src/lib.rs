//! GATOS Ledger (meta-crate)
//!
//! Feature-gated façade for the ledger. Select exactly one backend:
//! - `git2-backend` (default): includes the `git2`-based storage adapter.
//! - `core-only`: only the `no_std` core types/traits.
//!
//! The two features are mutually exclusive. Exactly one must be enabled.

#[cfg(all(feature = "core-only", feature = "git2-backend"))]
compile_error!("features `core-only` and `git2-backend` are mutually exclusive");

#[cfg(not(any(feature = "core-only", feature = "git2-backend")))]
compile_error!("enable exactly one of `core-only` or `git2-backend` features");

// Re-export the selected backend's public API.
//
// Rationale: This meta-crate is a façade; consumers should `use gatos_ledger::*`
// and not care which backend is selected. The underlying crates define a
// relatively wide surface (types + traits). Maintaining an explicit list here
// would be error-prone and create drift; using a glob keeps the façade aligned
// with the backend/core. Refer to backend crate docs for the detailed surface.
#[cfg(feature = "git2-backend")]
pub use gatos_ledger_git::*;

#[cfg(feature = "core-only")]
pub use gatos_ledger_core::*;
