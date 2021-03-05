pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("io error: {0}")]
	Io(#[from] std::io::Error),
	#[error("compression error: {0}")]
	Compression(String),
	#[error("failed to parse control file: {0}")]
	ControlSyntax(String),
	#[error("control file was missing {0}")]
	MissingPart(String),
	#[error("deb file was empty")]
	Empty,
}

impl<'a> From<debcontrol::SyntaxError<'a>> for Error {
	fn from(err: debcontrol::SyntaxError<'a>) -> Self {
		Error::ControlSyntax(err.to_string())
	}
}

impl From<lzma::LzmaError> for Error {
	fn from(err: lzma::LzmaError) -> Self {
		match err {
			lzma::LzmaError::Io(err) => Self::Io(err),
			_ => Self::Compression(err.to_string()),
		}
	}
}
