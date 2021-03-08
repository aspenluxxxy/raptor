/*
	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{ControlFile, Error, Result};
use ar::Archive as Ar;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use lzma::LzmaReader;
use std::{
	io::{BufRead, BufReader, Cursor, Read},
	path::Path,
};
use tar::{Archive as TarArchive, Builder as TarBuilder};
use zstd::Decoder as ZstdDecoder;

pub struct DebFile {
	pub debian_binary: String,
	pub control: TarArchive<Box<dyn Read + Send>>,
	pub data: TarArchive<Box<dyn Read + Send>>,
}

impl DebFile {
	pub fn parse(deb: Vec<u8>) -> Result<DebFile> {
		let deb = Cursor::new(deb);
		let mut debian_binary = None;
		let mut control = None;
		let mut data = None;
		let mut archive = Ar::new(deb);
		while let Some(Ok(mut entry)) = archive.next_entry() {
			let name = String::from_utf8_lossy(entry.header().identifier()).to_string();
			let mut contents = Vec::<u8>::with_capacity(entry.header().size() as usize);
			entry.read_to_end(&mut contents)?;
			if name == "debian-binary" {
				debian_binary = Some(String::from_utf8_lossy(&contents).trim().to_lowercase());
			} else if name.starts_with("control.tar") {
				control = Some(Self::read_control_file(&name, Cursor::new(contents))?);
			} else if name.starts_with("data.tar") {
				data = Some(Self::read_data(&name, Cursor::new(contents))?);
			}
		}
		Ok(Self {
			debian_binary: debian_binary
				.ok_or_else(|| Error::MissingPart("debian-binary".into()))?,
			control: control.ok_or_else(|| Error::MissingPart("control.tar".into()))?,
			data: data.ok_or_else(|| Error::MissingPart("data.tar".into()))?,
		})
	}

	pub fn debian_binary(&self) -> &str {
		&self.debian_binary
	}

	pub fn control(&mut self) -> Result<ControlFile> {
		for entry in self.control.entries()? {
			let entry = entry?;
			if entry
				.path()?
				.file_name()
				.and_then(|name| name.to_str())
				.map(|name| name == "control")
				.unwrap_or(false)
			{
				return ControlFile::parse(entry);
			}
		}
		Err(Error::MissingPart("control".into()))
	}

	#[doc(hidden)]
	pub(crate) fn boxed_control(&mut self) -> Result<Box<ControlFile>> {
		self.control().map(Box::new)
	}

	pub fn list_files(&mut self) -> Result<Vec<String>> {
		Ok(self
			.data
			.entries()?
			.filter_map(|entry| {
				let entry = entry.ok()?;
				let path = entry.path().ok()?;
				path.to_str().map(|s| s.to_string())
			})
			.collect())
	}

	pub fn unpack<P>(&mut self, destination: P) -> Result<()>
	where
		P: AsRef<Path>,
	{
		Ok(self.data.unpack(destination)?)
	}

	fn read_data(name: &str, entry: Cursor<Vec<u8>>) -> Result<TarArchive<Box<dyn Read + Send>>> {
		let reader = match name {
			"data.tar.gz" => Box::new(GzDecoder::new(entry)) as Box<dyn Read + Send>,
			"data.tar.xz" => Box::new(LzmaReader::new_decompressor(entry)?),
			"data.tar.bz2" => Box::new(BzDecoder::new(entry)),
			"data.tar.zst" => Box::new(ZstdDecoder::new(entry)?),
			_ => Box::new(entry),
		};
		Ok(TarArchive::new(Box::new(reader)))
	}

	fn read_control_file(
		name: &str,
		entry: Cursor<Vec<u8>>,
	) -> Result<TarArchive<Box<dyn Read + Send>>> {
		let reader = match name {
			"control.tar.gz" => Box::new(GzDecoder::new(entry)) as Box<dyn Read + Send>,
			"control.tar.xz" => Box::new(LzmaReader::new_decompressor(entry)?),
			"control.tar.zst" => Box::new(ZstdDecoder::new(entry)?),
			_ => Box::new(entry),
		};
		Ok(TarArchive::new(reader))
	}

	pub fn pack<A, B>(control: A, data: B) -> Result<Self>
	where
		A: AsRef<Path>,
		B: AsRef<Path>,
	{
		let control = {
			let control = control.as_ref();
			let tar = Cursor::new(Vec::<u8>::with_capacity(4096));
			let mut builder = TarBuilder::new(tar);
			builder.append_dir_all("/", control)?;
			builder.finish()?;
			TarArchive::new(
				Box::new(BufReader::new(builder.into_inner()?)) as Box<dyn BufRead + Send>
			)
		};
		let data = {
			let data = data.as_ref();
			let tar = Cursor::new(Vec::<u8>::with_capacity(4096));
			let mut builder = TarBuilder::new(tar);
			builder.append_dir_all("/", data)?;
			builder.finish()?;
			TarArchive::new(
				Box::new(BufReader::new(builder.into_inner()?)) as Box<dyn BufRead + Send>
			)
		};
		todo!()
	}
}
