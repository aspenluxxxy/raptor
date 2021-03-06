use raptor::apt::DebFile;
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
	let update_at = amt_files / 10;
	for (idx, file) in files.iter().enumerate() {
		let mut v = Vec::new();
		if idx % update_at == 0 {
			println!(
				"{}% done",
				((idx as f32 / amt_files as f32) * 100.0).round() as u8
			);
		}
		{
			let mut fd = BufReader::new(File::open(file).unwrap());
			fd.read_to_end(&mut v)
				.unwrap_or_else(|err| panic!("failed to read '{}': {:?}", file.display(), err));
		}
		DebFile::parse(&v)
			.unwrap_or_else(|err| panic!("failed to parse '{}': {:?}", file.display(), err));
	}
	let time = start.elapsed();
	println!(
		"took {} seconds to parse {} deb files",
		time.as_secs(),
		files.len()
	);
}
