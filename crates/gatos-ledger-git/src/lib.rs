//! Temporary stub for the git-backed ledger backend.
//!
//! This allows the workspace to build while the real implementation is rebuilt
//! according to ADR-0001 and the Ledger-Kernel comparison notes.

#![deny(unsafe_code)]

/// Returns a static string explaining that the backend is missing.
pub fn stub_notice() -> &'static str {
    "gatos-ledger-git backend is under reconstruction"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_notice_mentions_backend() {
        assert!(stub_notice().contains("backend"));
    }
}
