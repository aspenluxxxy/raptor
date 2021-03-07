/*
	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

mod args;

use raptor::DebFile;
use std::{
	fs::File,
	io::{BufReader, Read},
	time::Instant,
};

fn main() {
	//let args = std::env::args().collect::<Vec<String>>();
	//let path = args[1].as_str();

	let mut files = Vec::with_capacity(4096);

	for file in std::fs::read_dir("procursus-tests").unwrap() {
		let file = file.unwrap();
		if !file.file_name().to_string_lossy().ends_with(".deb") {
			continue;
		}
		files.push(file.path());
	}

	let start = Instant::now();
	let amt_files = files.len();
	for file in files {
		let mut v = Vec::new();
		let mut fd = BufReader::new(File::open(&file).unwrap());
		fd.read_to_end(&mut v)
			.unwrap_or_else(|err| panic!("failed to read '{}': {:?}", file.display(), err));
		DebFile::parse(&v)
			.unwrap_or_else(|err| panic!("failed to parse '{}': {:?}", file.display(), err));
	}
	let time = start.elapsed();
	println!(
		"took {:.4} seconds to parse {} deb files",
		time.as_millis() as f64 / 1000.0,
		amt_files
	);
}
