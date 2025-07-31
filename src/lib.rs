pub mod bzip;
pub mod c77;

pub mod ed6;
pub mod ed7;

mod util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error("failed to read at position {pos}")]
	Read {
		pos: usize,
	},
	#[error("attempted to repeat {count} bytes from offset -{offset}, but only have {len} bytes")]
	BadRepeat {
		count: usize,
		offset: usize,
		len: usize,
	},
	#[error("wrong {what}: expected {expected}, got {actual}")]
	BadSize { what: &'static str, expected: usize, actual: usize },
	#[error("{message}")]
	Custom { message: String },
}

impl From<gospel::read::Error> for Error {
	fn from(e: gospel::read::Error) -> Self {
		Error::Read { pos: e.pos() }
	}
}

impl Error {
	fn check_size(what: &'static str, expected: usize, actual: usize) -> Result<()> {
		if expected == actual {
			Ok(())
		} else {
			Err(Error::BadSize { what, expected, actual })
		}
	}
}

pub type Result<A, E = Error> = std::result::Result<A, E>;
