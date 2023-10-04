pub mod bzip;
pub mod c77;

pub mod util;

#[derive(Debug, thiserror::Error)]
pub enum Error {
	#[error(transparent)]
	Read { #[from] source: gospel::read::Error },
	#[error("attempted to repeat {count} bytes from offset -{offset}, but only have {len} bytes")]
	BadRepeat {
		count: usize,
		offset: usize,
		len: usize,
	},
	#[error("invalid frame")]
	Frame,
}

impl Error {
	fn check_size(expected: usize, actual: usize) -> Result<()> {
		if expected == actual {
			Ok(())
		} else {
			Err(Error::Frame)
		}
	}
}

pub type Result<A, E=Error> = std::result::Result<A, E>;
