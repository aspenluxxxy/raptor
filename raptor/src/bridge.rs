use crate::{Compression, ControlEntry, ControlFile, DebFile, Result};

#[cxx::bridge(namespace = "raptor")]
mod ffi {
	enum Compression {
		Bz2,
		Gz,
		Xz,
		Zst,
	}
	extern "Rust" {
		type DebFile;
		fn parse_deb(deb: Vec<u8>) -> Result<Box<DebFile>>;
		fn debian_binary(&self) -> &str;
		fn unpack(&mut self, destination: &str) -> Result<()>;
		fn list_files(&mut self) -> Result<Vec<String>>;
		#[rust_name = "boxed_control"]
		fn control(self: &mut DebFile) -> Result<Box<ControlFile>>;
		#[rust_name = "deb_boxed_pack"]
		fn pack(control: &str, data: &str) -> Result<Box<DebFile>>;
		#[rust_name = "deb_write_to_file"]
		fn write(deb: Box<DebFile>, destination: &str, compression: Compression) -> Result<()>;
	}
	extern "Rust" {
		type ControlEntry;
		fn entry_value(value: String) -> Box<ControlEntry>;
		fn entry_multivalue(value: Vec<String>) -> Box<ControlEntry>;
		fn entry_number(value: u64) -> Box<ControlEntry>;
		fn entry_bool(value: bool) -> Box<ControlEntry>;
		fn entry_from_yesno(value: &str) -> Box<ControlEntry>;
		fn entry_from_commalist(value: &str) -> Box<ControlEntry>;
		fn entry_from_number(value: &str) -> Box<ControlEntry>;
		fn to_string(&self) -> String;
	}
	extern "Rust" {
		type ControlFile;
		fn parse_controlfile(deb: &[u8]) -> Result<Box<ControlFile>>;
		fn parse_controlfile_multi(data: &[u8]) -> Result<Vec<ControlFile>>;
		fn to_string(&self) -> String;
	}
}

fn parse_deb(deb: Vec<u8>) -> Result<Box<DebFile>> {
	DebFile::parse(deb).map(Box::new)
}

fn deb_write_to_file(
	deb: Box<DebFile>,
	destination: &str,
	compression: ffi::Compression,
) -> Result<()> {
	let compression = match compression {
		ffi::Compression::Bz2 => Compression::Bz2,
		ffi::Compression::Gz => Compression::Gz,
		ffi::Compression::Xz => Compression::Xz,
		ffi::Compression::Zst => Compression::Zst,
		_ => Compression::Xz,
	};
	deb.write(std::fs::File::create(destination)?, compression)
}

fn deb_boxed_pack(control: &str, data: &str) -> Result<Box<DebFile>> {
	Ok(Box::new(DebFile::pack(control, data)?))
}

fn parse_controlfile(data: &[u8]) -> Result<Box<ControlFile>> {
	ControlFile::parse(data).map(Box::new)
}

fn parse_controlfile_multi(data: &[u8]) -> Result<Vec<ControlFile>> {
	ControlFile::parse_multi(data)
}

fn entry_value(value: String) -> Box<ControlEntry> {
	Box::new(ControlEntry::Value(value))
}

fn entry_multivalue(value: Vec<String>) -> Box<ControlEntry> {
	Box::new(ControlEntry::MultiValue(value))
}

fn entry_number(value: u64) -> Box<ControlEntry> {
	Box::new(ControlEntry::Number(value))
}

fn entry_bool(value: bool) -> Box<ControlEntry> {
	Box::new(ControlEntry::Bool(value))
}

fn entry_from_yesno(value: &str) -> Box<ControlEntry> {
	Box::new(ControlEntry::from_yesno(value))
}

fn entry_from_commalist(value: &str) -> Box<ControlEntry> {
	Box::new(ControlEntry::from_commalist(value))
}

fn entry_from_number(value: &str) -> Box<ControlEntry> {
	Box::new(ControlEntry::from_number(value))
}
