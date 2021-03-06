use super::control::ControlFile;
use crate::{Error, Result};
use ar::{Archive as Ar, Entry as ArEntry};
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lzma::LzmaReader;
use std::{
	fmt,
	io::{BufRead, BufReader, Cursor, Read},
};
use tar::Archive as TarArchive;
use zstd::Decoder as ZstdDecoder;

pub struct DebFile<'a> {
	pub debian_binary: String,
	pub control: ControlFile,
	pub data: TarArchive<Box<dyn BufRead + 'a>>,
}

impl<'a> fmt::Debug for DebFile<'a> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("DebFile")
			.field("debian_binary", &self.debian_binary)
			.field("control", &self.control)
			.finish()
	}
}

impl<'a> DebFile<'a> {
	pub fn parse(deb: &'a [u8]) -> Result<DebFile<'a>> {
		let deb = Cursor::new(deb);
		let mut debian_binary = None;
		let mut control = None;
		let mut data = None;
		let mut archive = Ar::new(deb);
		while let Some(Ok(mut entry)) = archive.next_entry() {
			let name = String::from_utf8_lossy(entry.header().identifier()).to_string();
			if name == "debian-binary" {
				let mut s = String::new();
				entry.read_to_string(&mut s)?;
				s.truncate(s.trim_end().len());
				debian_binary = Some(s);
			} else if name.starts_with("control.tar") {
				control = Some(Self::read_control_file(&name, &mut entry)?);
			} else if name.starts_with("data.tar") {
				data = Some(Self::read_data(&name, &mut entry)?);
			}
		}
		Ok(Self {
			debian_binary: debian_binary
				.ok_or_else(|| Error::MissingPart("debian-binary".into()))?,
			control: control.ok_or_else(|| Error::MissingPart("control".into()))?,
			data: data.ok_or_else(|| Error::MissingPart("data".into()))?,
		})
	}

	fn read_data(
		name: &str,
		entry: &'a mut ArEntry<'a, Cursor<&'a [u8]>>,
	) -> Result<TarArchive<Box<dyn BufRead + 'a>>> {
		let reader = match name {
			"data.tar.gz" => Box::new(GzDecoder::new(entry)) as Box<dyn Read + 'a>,
			"data.tar.xz" => Box::new(LzmaReader::new_decompressor(entry)?),
			"data.tar.bz2" => Box::new(BzDecoder::new(entry)),
			"data.tar.zst" => Box::new(ZstdDecoder::new(entry)?),
			_ => {
				todo!()
			}
		};
		Ok(TarArchive::new(Box::new(BufReader::new(reader))))
	}

	fn read_control_file(name: &str, entry: &mut ArEntry<Cursor<&[u8]>>) -> Result<ControlFile> {
		let reader = match name {
			"control.tar.gz" => Box::new(GzDecoder::new(entry)) as Box<dyn Read>,
			"control.tar.xz" => Box::new(LzmaReader::new_decompressor(entry)?) as Box<dyn Read>,
			"control.tar.zst" => Box::new(ZstdDecoder::new(entry)?) as Box<dyn Read>,
			_ => Box::new(entry),
		};
		let mut archive = TarArchive::new(reader);
		let control_entry = archive
			.entries()?
			.flatten()
			.find(|entry| {
				entry
					.path()
					.map(|path| path.ends_with("control"))
					.unwrap_or(false)
			})
			.ok_or_else(|| Error::MissingPart("control".into()))?;
		ControlFile::parse(BufReader::new(control_entry))
	}
}
