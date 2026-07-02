//! Lab crate plumbing — you never need to edit this file.
//!
//! Your work happens in `src/lab.rs`. With `--features solutions` the crate
//! re-exports the reference implementations instead, which is how
//! `./grade verify` proves every stage is passable.

#[cfg(not(feature = "solutions"))]
mod lab;
#[cfg(not(feature = "solutions"))]
pub use lab::*;

#[cfg(feature = "solutions")]
pub use taocp_reference::m20_networks::*;
