#![deny(
	clippy::complexity,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	unsafe_code
)]

#[macro_use]
extern crate thiserror;

pub mod archive;
pub mod control;
pub mod error;

pub use archive::DebFile;
pub use control::{ControlEntry, ControlFile};
pub use error::{Error, Result};
