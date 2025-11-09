//! GATOS Ledger (meta-crate)
//!
//! Feature-gated façade for the ledger. Select exactly one backend:
//! - `git2-backend` (default): includes the `git2`-based storage adapter.
//! - `core-only`: only the `no_std` core types/traits.
//!
//! The two features are mutually exclusive.

#[cfg(all(feature = "core-only", feature = "git2-backend"))]
compile_error!("features `core-only` and `git2-backend` are mutually exclusive");

#[cfg(feature = "git2-backend")]
pub use gatos_ledger_git::*;

#[cfg(feature = "core-only")]
pub use gatos_ledger_core::*;
