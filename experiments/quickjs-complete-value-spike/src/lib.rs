#![deny(unsafe_code)]
//! Isolated §6 feasibility, not a production runtime or operation schema validator.
mod inspect;
mod transfer;
pub use transfer::{Boundary, Failure, Limits, Portable, Usage};
#[cfg(test)]
mod tests;
