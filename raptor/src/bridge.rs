use crate::{ControlEntry, ControlFile, DebFile, Result};

#[cxx::bridge(namespace = "raptor")]
mod ffi {
	extern "Rust" {
		type DebFile;
		fn parse_deb(deb: &[u8]) -> Result<Box<DebFile>>;
	}
	extern "Rust" {
		type ControlEntry;
		fn entry_value(value: String) -> Box<ControlEntry>;
		fn entry_multivalue(value: Vec<String>) -> Box<ControlEntry>;
		fn entry_number(value: u64) -> Box<ControlEntry>;
		fn entry_bool(value: bool) -> Box<ControlEntry>;
		fn to_string(&self) -> String;
	}
	extern "Rust" {
		type ControlFile;
		fn parse_controlfile(deb: &[u8]) -> Result<Box<ControlFile>>;
		fn parse_controlfile_multi(data: &[u8]) -> Result<Vec<ControlFile>>;
		fn to_string(&self) -> String;
	}
}

fn parse_deb(deb: &[u8]) -> Result<Box<DebFile>> {
	DebFile::parse(deb).map(Box::new)
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
