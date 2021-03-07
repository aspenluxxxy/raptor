/*
	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

mod args;

use crate::args::CmdArgs;
use raptor::{ControlEntry, DebFile};
use rayon::iter::{ParallelBridge, ParallelIterator};
use sha2::{Digest, Sha256};
use std::{
	fs::File,
	io::{BufReader, Read, Write},
};
use structopt::StructOpt;

fn main() {
	let args = CmdArgs::from_args();
	match args {
		CmdArgs::Scan { folder, .. } => {
			let dir = std::fs::read_dir(&folder).expect("failed to scan directory");
			let controls = dir
				.par_bridge()
				.map(|entry| {
					let entry = entry.expect("failed to get directory entry");
					let mut contents = Vec::<u8>::with_capacity(32768);
					let mut file =
						BufReader::new(File::open(entry.path()).expect("failed to open file"));
					file.read_to_end(&mut contents)
						.expect("failed to read file");
					let deb = DebFile::parse(&contents).expect("failed to parse deb file");
					let mut control = deb.control;
					let mut sha = Sha256::default();
					sha.update(&contents);
					control.insert(
						"SHA256".to_string(),
						ControlEntry::Value(hex::encode(sha.finalize().to_vec())),
					);
					control.to_string()
				})
				.collect::<Vec<_>>();
			let stdout = std::io::stdout();
			writeln!(stdout.lock(), "{}", controls.join("\n")).expect("failed to write to stdout");
		}
		_ => unimplemented!(),
	}
}
