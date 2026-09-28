#![deny(unsafe_code)]
//! Non-production engine-class gate, not RFC value validation or copying.

mod bridge;
pub use bridge::{ClassBridge, ForeignContext, ObjectClass};

#[cfg(test)]
mod tests;
