#![allow(unreachable_patterns)]

pub mod crypto;
pub mod mock;
pub mod node;
pub mod obj;
#[cfg(test)]
mod tests;
mod utils;

pub const CURRENT_VERSION: u32 = 0;
