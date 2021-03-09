/*
	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{compression::Compression, ControlFile, Error, Result};
use ar::{Archive as Ar, Builder as ArBuilder, Header as ArHeader};
use std::{
	io::{BufRead, BufReader, Cursor, Read, Write},
	path::Path,
	str::FromStr,
};
use tar::{Archive as TarArchive, Builder as TarBuilder};

pub struct DebFile {
	pub debian_binary: String,
	pub control: TarArchive<Box<dyn BufRead + Send>>,
	pub data: TarArchive<Box<dyn BufRead + Send>>,
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

	fn read_data(
		name: &str,
		entry: Cursor<Vec<u8>>,
	) -> Result<TarArchive<Box<dyn BufRead + Send>>> {
		let entry = Box::new(BufReader::new(entry));
		let decompressor = Box::new(BufReader::new(
			Compression::from_str(name)?.decompress(entry)?,
		));
		Ok(TarArchive::new(decompressor))
	}

	fn read_control_file(
		name: &str,
		entry: Cursor<Vec<u8>>,
	) -> Result<TarArchive<Box<dyn BufRead + Send>>> {
		let entry = Box::new(BufReader::new(entry));
		let decompressor = Box::new(BufReader::new(
			Compression::from_str(name)?.decompress(entry)?,
		));
		Ok(TarArchive::new(decompressor))
	}

	pub fn pack<A, B>(control: A, data: B) -> Result<Self>
	where
		A: AsRef<Path>,
		B: AsRef<Path>,
	{
		let control = {
			let control = control.as_ref();
			let tar = {
				let tar = Vec::<u8>::new();
				let mut builder = TarBuilder::new(tar);
				builder.append_dir_all("", control)?;
				Cursor::new(builder.into_inner()?)
			};
			let reader = Box::new(BufReader::new(tar)) as Box<dyn BufRead + Send>;
			TarArchive::new(reader)
		};
		let data = {
			let data = data.as_ref();
			let tar = {
				let tar = Vec::<u8>::new();
				let mut builder = TarBuilder::new(tar);
				builder.append_dir_all("", data)?;
				Cursor::new(builder.into_inner()?)
			};
			let reader = Box::new(BufReader::new(tar)) as Box<dyn BufRead + Send>;
			TarArchive::new(reader)
		};
		Ok(Self {
			debian_binary: "2.0\n".into(),
			control,
			data,
		})
	}

	pub fn write<W: Write>(self, destination: W, compression: Compression) -> Result<()> {
		let debian_binary_name = b"debian-binary".to_vec();
		let control_name = format!("control.tar.{}", compression).as_bytes().to_vec();
		let data_name = format!("data.tar.{}", compression).as_bytes().to_vec();
		// Set up compression
		let mut control = compression.compress(self.control.into_inner())?;
		let mut data = compression.compress(self.data.into_inner())?;
		// Alright, time to start building our archive
		let mut builder = ArBuilder::new(destination);
		// Create debian-binary
		let header = ArHeader::new(debian_binary_name, 4);
		builder.append(&header, self.debian_binary.as_bytes())?;
		// Create control.tar
		let mut control_bytes = Vec::<u8>::new();
		control.read_to_end(&mut control_bytes)?;
		let header = ArHeader::new(control_name, control_bytes.len() as u64);
		builder.append(&header, &control_bytes as &[u8])?;
		// Create data.tar
		let mut data_bytes = Vec::<u8>::new();
		data.read_to_end(&mut data_bytes)?;
		let header = ArHeader::new(data_name, data_bytes.len() as u64);
		builder.append(&header, &data_bytes as &[u8])?;
		// Alright, we're done, ensure everything's flushed.
		builder.into_inner()?.flush()?;
		Ok(())
	}
}
