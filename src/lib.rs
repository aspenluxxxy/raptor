/*
	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

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
