use super::control::ControlFile;
use ar::{Archive as Ar, Entry as ArEntry};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lzma_rs::xz_decompress;
use std::{
	fmt,
	io::{BufRead, BufReader, Cursor, Read},
};
use tar::Archive as TarArchive;
use zstd::Decoder as ZstdDecoder;

pub struct DebFile {
	pub debian_binary: String,
	pub control: ControlFile,
	pub data: TarArchive<Box<dyn BufRead>>,
}

impl fmt::Debug for DebFile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("DebFile")
			.field("debian_binary", &self.debian_binary)
			.field("control", &self.control)
			.finish()
	}
}

impl DebFile {
	pub fn new(deb: Vec<u8>) -> Self {
		let deb = Cursor::new(deb);
		let mut debian_binary = None;
		let mut control = None;
		let mut data = None;
		let mut archive = Ar::new(deb);
		while let Some(Ok(mut entry)) = archive.next_entry() {
			let name = String::from_utf8_lossy(entry.header().identifier()).to_string();
			if name == "debian-binary" {
				let mut s = String::new();
				entry.read_to_string(&mut s).unwrap();
				s.truncate(s.trim_end().len());
				debian_binary = Some(s);
			} else if name.starts_with("control.tar") {
				control = Some(Self::read_control_file(&name, entry));
			} else if name.starts_with("data.tar") {
				data = Some(Self::read_data(&name, entry));
			}
		}
		Self {
			debian_binary: debian_binary.unwrap(),
			control: control.unwrap(),
			data: data.unwrap(),
		}
	}

	fn read_data(name: &str, mut entry: ArEntry<Cursor<Vec<u8>>>) -> TarArchive<Box<dyn BufRead>> {
		let mut decompressed = Vec::<u8>::new();
		let reader = match name {
			"data.tar.gz" => {
				let mut decoder = GzDecoder::new(entry);
				decoder.read_to_end(&mut decompressed).unwrap();
				Box::new(Cursor::new(decompressed)) as Box<dyn Read>
			}
			"data.tar.xz" => {
				let mut entry = BufReader::new(entry);
				xz_decompress(&mut entry, &mut decompressed).unwrap();
				Box::new(Cursor::new(decompressed))
			}
			"data.tar.bz2" => {
				let mut decoder = BzDecoder::new(entry);
				decoder.read_to_end(&mut decompressed).unwrap();
				Box::new(Cursor::new(decompressed))
			}
			"data.tar.zst" => {
				let mut decoder = ZstdDecoder::new(entry).unwrap();
				decoder.read_to_end(&mut decompressed).unwrap();
				Box::new(Cursor::new(decompressed))
			}
			_ => {
				std::io::copy(&mut entry, &mut decompressed).unwrap();
				Box::new(Cursor::new(decompressed))
			}
		};
		TarArchive::new(Box::new(BufReader::new(reader)))
	}

	fn read_control_file(name: &str, entry: ArEntry<Cursor<Vec<u8>>>) -> ControlFile {
		let reader = match name {
			"control.tar.gz" => Box::new(GzDecoder::new(entry)) as Box<dyn Read>,
			"control.tar.xz" => {
				let mut decompressed = Vec::<u8>::new();
				let mut entry = BufReader::new(entry);
				xz_decompress(&mut entry, &mut decompressed).unwrap();
				Box::new(Cursor::new(decompressed))
			}
			"control.tar.zst" => Box::new(ZstdDecoder::new(entry).unwrap()) as Box<dyn Read>,
			_ => Box::new(entry),
		};
		let mut archive = TarArchive::new(reader);
		let control_entry = archive
			.entries()
			.unwrap()
			.flatten()
			.find(|entry| {
				entry
					.path()
					.map(|path| path.ends_with("control"))
					.unwrap_or(false)
			})
			.unwrap();
		ControlFile::new(BufReader::new(control_entry))
	}
}
