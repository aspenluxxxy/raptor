use raptor::compression::Compression;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum CmdArgs {
	/// Pack a directory into a .deb file
	/// This is equivalent to dpkg-deb -b.
	Pack {
		/// Compress the .deb file using the given compression value.
		/// Valid values: bz2, gz, xz, zstd
		#[structopt(short = "x", long = "compress", default_value = "xz")]
		compression: Compression,
		/// The directory that contains the deb's control file
		/// and maintainer scripts
		#[structopt(short, long, parse(from_os_str))]
		control: PathBuf,
		/// The main input folder, containing the files to
		/// install with the deb.
		#[structopt(short, long, parse(from_os_str))]
		input: PathBuf,
		/// The location where the newly created deb file
		/// will be written to.
		#[structopt(short, long, parse(from_os_str))]
		output: PathBuf,
	},
	/// Unpack the contents of a .deb file.
	/// This is equivalent to dpkg-deb -x.
	Unpack {
		/// The input deb file that will be unpacked.
		#[structopt(short, long, parse(from_os_str))]
		input: PathBuf,
		/// The folder to unpack the deb file into.
		#[structopt(short, long, parse(from_os_str))]
		output: PathBuf,
	},
	/// Scan a folder, creating a Packages folder from the debs inside.
	/// This is equivalent to dpkg-scanpackages.
	Scan {
		/// The prefix to use for all URLs in the resulting Packages file
		#[structopt(short, long)]
		prefix: Option<String>,
		/// The folder to scan .deb files from.
		#[structopt(short, long, parse(from_os_str))]
		input: PathBuf,
	},
}
