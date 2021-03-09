use raptor::compression::Compression;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum CmdArgs {
	Pack {
		#[structopt(short = "c", long = "compress", default_value = "xz")]
		compression: Compression,
		#[structopt(short, long, parse(from_os_str))]
		control: PathBuf,
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
