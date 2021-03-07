/*
	This Source Code Form is subject to the terms of the Mozilla Public
	License, v. 2.0. If a copy of the MPL was not distributed with this
	file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

use crate::{Error, Result};
use std::{
	collections::BTreeMap,
	fmt,
	io::BufRead,
	iter::FromIterator,
	ops::{Deref, DerefMut},
};

#[derive(Debug, Clone)]
pub enum ControlEntry {
	Value(String),
	MultiValue(Vec<String>),
	Number(u64),
	Bool(bool),
}

impl fmt::Display for ControlEntry {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let x = match self {
			ControlEntry::Value(s) => s.to_string(),
			ControlEntry::MultiValue(v) => v.join(", "),
			ControlEntry::Number(n) => n.to_string(),
			ControlEntry::Bool(b) => {
				if *b {
					"Yes".to_string()
				} else {
					"No".to_string()
				}
			}
		};
		write!(f, "{}", x)?;
		Ok(())
	}
}

impl ControlEntry {
	pub fn from_yesno(d: &str) -> ControlEntry {
		if d == "yes" {
			ControlEntry::Bool(true)
		} else {
			ControlEntry::Bool(false)
		}
	}

	pub fn from_commalist(d: &str) -> ControlEntry {
		ControlEntry::MultiValue(d.split(',').map(|x| x.trim().to_string()).collect())
	}

	pub fn from_number(d: &str) -> ControlEntry {
		ControlEntry::Number(d.parse::<u64>().unwrap_or(0))
	}
}

#[derive(Debug, Clone)]
pub struct ControlFile(BTreeMap<String, ControlEntry>);

impl ControlFile {
	pub fn parse<T: BufRead>(data: T) -> Result<ControlFile> {
		Self::parse_multi(data).and_then(|parsed| parsed.first().cloned().ok_or(Error::Empty))
	}

	pub fn parse_multi<T: BufRead>(mut data: T) -> Result<Vec<ControlFile>> {
		let mut parsed = Vec::with_capacity(1);
		let mut buf = String::new();
		data.read_to_string(&mut buf)?;
		let paragraphs = debcontrol::parse_str(&buf)?;
		for paragraph in paragraphs {
			let mut map: BTreeMap<String, ControlEntry> = BTreeMap::default();
			for field in &paragraph.fields {
				let (key, value) = (field.name.trim(), field.value.trim());
				let entry = match key.to_lowercase().as_str() {
					"installed-size" => ControlEntry::from_number(value),
					"essential" | "build-essential" => ControlEntry::from_yesno(value),
					"tag" | "depends" | "pre-depends" | "recommends" | "suggests" | "enhances"
					| "build-depends" | "breaks" | "conflicts" | "provides" | "replaces"
					| "built-using" => ControlEntry::from_commalist(value),
					_ => ControlEntry::Value(value.into()),
				};
				map.insert(key.into(), entry);
			}
			parsed.push(Self(map));
		}
		Ok(parsed)
	}

	fn sort_keys(&self) -> Vec<(&String, &ControlEntry)> {
		let mut v = Vec::from_iter(&self.0);
		v.sort_by_key(|&(k, _)| k.as_str() == "Package");
		v.sort_by_key(|&(k, _)| k.as_str() == "Version");
		v.sort_by_key(|&(k, _)| k.as_str() == "Name");
		v.sort_by_key(|&(k, _)| k.as_str() == "Author");
		v.sort_by_key(|&(k, _)| k.as_str() == "Maintainer");
		v.sort_by_key(|&(k, _)| k.as_str() == "MD5sum");
		v.sort_by_key(|&(k, _)| k.as_str() == "SHA1");
		v.sort_by_key(|&(k, _)| k.as_str() == "SHA256");
		v.sort_by_key(|&(k, _)| {
			k.as_str() != "Version"
				&& k.as_str() != "Package"
				&& k.as_str() != "Name"
				&& k.as_str() != "Author"
				&& k.as_str() != "Maintainer"
				&& k.as_str() != "MD5sum"
				&& k.as_str() != "SHA1"
				&& k.as_str() != "SHA256"
		});
		v
	}
}

impl fmt::Display for ControlFile {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for &(key, value) in &self.sort_keys() {
			writeln!(f, "{}: {}", key, value.to_string().replace('\n', "\n "))?;
		}
		Ok(())
	}
}

impl Deref for ControlFile {
	type Target = BTreeMap<String, ControlEntry>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for ControlFile {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
