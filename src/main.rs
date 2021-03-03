use raptor::apt::archive::DebFile;

fn main() {
	let args = std::env::args().collect::<Vec<String>>();
	let path = args[1].as_str();

	let deb = DebFile::new(std::fs::read(path).unwrap());
	println!("{:#?}", deb);
}
