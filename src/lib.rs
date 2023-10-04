pub mod bzip;
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

pub type Result<A, E=Error> = std::result::Result<A, E>;
