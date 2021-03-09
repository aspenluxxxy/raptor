use crate::{Error, Result};
use bzip2::read::{BzDecoder, BzEncoder};
use flate2::read::{GzDecoder, GzEncoder};
use lzma::LzmaReader;
use std::{
	fmt::{self, Display},
	io::{BufRead, Read},
	str::FromStr,
};
use zstd::stream::read::{Decoder as ZstdDecoder, Encoder as ZstdEncoder};

pub enum Compression {
	/// Bzip compression
	Bz2,
	/// Gzip compression
	Gz,
	/// LZMA / XZ compression
	Xz,
	/// Zstandard compression
	Zst,
}

impl Compression {
	/// Create a compression stream using the given buffered reader
	pub fn compress(&self, reader: Box<dyn BufRead + Send>) -> Result<Box<dyn Read + Send>> {
		match self {
			Compression::Bz2 => Ok(Box::new(BzEncoder::new(
				reader,
				bzip2::Compression::default(),
			))),
			Compression::Gz => Ok(Box::new(GzEncoder::new(
				reader,
				flate2::Compression::default(),
			))),
			Compression::Xz => Ok(Box::new(LzmaReader::new_compressor(reader, 6)?)),
			Compression::Zst => Ok(Box::new(ZstdEncoder::new(reader, 6)?)),
		}
	}

	/// Create a decompression stream using the given buffered reader
	pub fn decompress(&self, reader: Box<dyn BufRead + Send>) -> Result<Box<dyn Read + Send>> {
		match self {
			Compression::Bz2 => Ok(Box::new(BzDecoder::new(reader))),
			Compression::Gz => Ok(Box::new(GzDecoder::new(reader))),
			Compression::Xz => Ok(Box::new(LzmaReader::new_decompressor(reader)?)),
			Compression::Zst => Ok(Box::new(ZstdDecoder::new(reader)?)),
		}
	}
}

impl FromStr for Compression {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		let s = s.trim().to_lowercase();
		if s.ends_with("bz2") {
			Ok(Compression::Bz2)
		} else if s.ends_with("gz") {
			Ok(Compression::Gz)
		} else if s.ends_with("xz") {
			Ok(Compression::Xz)
		} else if s.ends_with("zst") || s.ends_with("zstd") {
			Ok(Compression::Zst)
		} else {
			Err(Error::InvalidCompression(s))
		}
	}
}

impl Display for Compression {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Compression::Bz2 => write!(f, "bz2"),
			Compression::Gz => write!(f, "gz"),
			Compression::Xz => write!(f, "xz"),
			Compression::Zst => write!(f, "zst"),
		}
	}
}
