//! EXP-0005 protocol-22 infrastructure. No sockets, authentication enforcement or domain adapters.
//! Bare dispatch is table-local; use one shared Registry for served cross-table effects.
pub mod codec;
mod connection;
mod dispatch;
pub mod framing;
mod query;
mod store;
mod types;
pub use connection::{Registry, handle_connection};
pub use dispatch::{dispatch, err_response, error_message};
pub use query::{
    default_relation_descriptors, page_by_scan, page_ids, page_key, page_key_value, page_rows,
    predicate_matches, validate_join, validate_predicate, validate_query,
};
pub use store::*;
pub use types::*;

#[cfg(test)]
extern crate self as uc_protocol;
