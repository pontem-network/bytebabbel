use std::io::Error as IoError;


#[derive(Debug)]
pub enum Error {
	InvalidHexCharacter,
	TooFewBytesForPush,
	Io(IoError),
}

impl std::error::Error for Error {}


impl From<IoError> for Error {
	fn from(err: IoError) -> Self { Error::Io(err) }
}

impl From<hex::FromHexError> for Error {
	fn from(_: hex::FromHexError) -> Self { Error::InvalidHexCharacter }
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidHexCharacter => write!(f, "Invalid hex character."),
			Self::TooFewBytesForPush => write!(f, "Too few bytes availabe to parse push operation."),
			Self::Io(err) => write!(f, "IO error: {}.", err),
		}
	}
}
