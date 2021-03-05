#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	unsafe_code
)]

#[macro_use]
extern crate thiserror;

pub mod apt;
pub mod error;

pub use error::{Error, Result};
