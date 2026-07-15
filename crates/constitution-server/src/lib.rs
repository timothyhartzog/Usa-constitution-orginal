//! HTTP surface for the constitution archive, factored out of `main.rs` so
//! integration tests can build the router against an in-memory archive.

#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::expect_used))]

pub mod error;
pub mod routes;
pub mod state;

#[cfg(feature = "ml")]
pub mod rag_types;

pub use error::ApiError;
pub use routes::router;
pub use state::AppState;
