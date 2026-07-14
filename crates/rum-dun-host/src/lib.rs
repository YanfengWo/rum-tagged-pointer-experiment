#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Dependency-free codec primitives for rum's Dun Plugin Protocol host.
//!
//! This crate currently contains only framed I/O, a bounded minimal JSON
//! codec, and typed protocol message conversion. The executable host and rum
//! integration are layered on top separately.

/// Length-prefixed protocol framing.
pub mod frame;
/// Minimal ordered JSON values and their codec.
pub mod json;
/// Typed inbound messages and outbound protocol builders.
pub mod protocol;
