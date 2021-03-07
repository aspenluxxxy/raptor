use std::{path::PathBuf, str::FromStr};
use structopt::StructOpt;

pub enum Compression {
	Lzma,
	Gzip,
	Zstd,
}

impl FromStr for Compression {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().trim() {
			"lzma" | "xz" => Ok(Compression::Lzma),
			"gz" | "gzip" => Ok(Compression::Gzip),
			"zstd" => Ok(Compression::Zstd),
			_ => Err("invalid compression type: expected lzma/xz, gz/gzip, or zstd.".into()),
		}
	}
}

#[derive(StructOpt)]
pub enum CmdArgs {
	Pack {
		#[structopt(short = "c", long = "compress", default_value = "Compression::Lzma")]
		compression: Compression,
		#[structopt(parse(from_os_str))]
		input: PathBuf,
		#[structopt(parse(from_os_str))]
		output: PathBuf,
	},
	Unpack {
		#[structopt(parse(from_os_str))]
		input: PathBuf,
		#[structopt(parse(from_os_str))]
		output: PathBuf,
	},
	Scan {
		#[structopt(parse(from_os_str))]
		folder: PathBuf,
		#[structopt(parse(from_os_str))]
		target: Option<PathBuf>,
	},
}
