//! Placeholder GraphQL gateway crate.
//!
//! This crate will eventually expose the `POST /api/v1/graphql` endpoint
//! described in ADR-0007. Right now it only defines struct stubs so other
//! crates can start wiring dependencies without the server existing yet.

#[cfg(feature = "server")]
pub mod api {
    use serde::{Deserialize, Serialize};

    /// Parameters accepted by the GraphQL endpoint.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GraphQlRequest {
        pub query: String,
        #[serde(default)]
        pub variables: serde_json::Value,
        #[serde(default)]
        pub operation_name: Option<String>,
        #[serde(default)]
        pub state_ref: Option<String>,
        #[serde(default)]
        pub ref_path: Option<String>,
    }

    /// Standard GraphQL response envelope.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GraphQlResponse {
        pub data: Option<serde_json::Value>,
        #[serde(default)]
        pub errors: Vec<serde_json::Value>,
        #[serde(default)]
        pub state_ref_resolved: Option<String>,
        #[serde(default)]
        pub shape_root: Option<String>,
    }
}

/// Minimal marker trait so downstream crates can depend on this crate before
/// the real server lands.
pub trait GraphQlService {
    /// Executes a GraphQL request and returns the JSON response body.
    fn execute(&self, request: &str) -> Result<String, GraphQlError>;
}

/// Placeholder error type.
#[derive(Debug, thiserror::Error)]
pub enum GraphQlError {
    #[error("not yet implemented")]
    NotImplemented,
}
