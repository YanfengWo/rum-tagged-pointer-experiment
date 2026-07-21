//! Optional Jupyter adapter and transport layer for `rum`.
//!
//! Implements the Jupyter messaging protocol over ZeroMQ, bridging a
//! [`rum_host`] session to Jupyter front ends (execute requests, completion,
//! inspection, and result MIME bundles). This crate is optional and is the only
//! front end that pulls in the async/transport stack (`tokio`, `zeromq`).

mod adapter;
mod bootstrap;
mod connection;
mod json_support;
mod launcher;
mod model;
mod signature;
mod socket;
mod transport;
mod zeromq;

pub use adapter::*;
pub use bootstrap::*;
pub use connection::*;
pub use launcher::*;
pub use model::*;
pub use socket::*;
pub use transport::*;
pub use zeromq::*;

#[cfg(test)]
mod tests;
