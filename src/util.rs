use std::iter::zip;

use crate::{Result, Error, offset_vec::OffsetVec};

pub fn count_equal(a: &[u8], b: &[u8], limit: usize) -> usize {
	let n = limit.min(a.len()).min(b.len());
	const N: usize = 8;

	let mut i = 0;
	for (a, b) in zip(a[..n].chunks_exact(N), b[..n].chunks_exact(N)) {
		if a == b {
			i += N;
		} else {
			let a = u64::from_le_bytes(a.try_into().unwrap());
			let b = u64::from_le_bytes(b.try_into().unwrap());
			return i + ((a ^ b).trailing_zeros() / 8) as usize;
		}
	}

	i = n.saturating_sub(N);
	zip(&a[i..n], &b[i..n])
		.take_while(|(a, b)| a == b)
		.count() + i
}

pub trait Output {
	fn constant(&mut self, count: usize, value: u8);
	fn verbatim(&mut self, s: &[u8]);
	fn repeat(&mut self, count: usize, offset: usize) -> Result<()>;
}

impl<'a> Output for &'a mut Vec<u8> {
	fn constant(&mut self, count: usize, value: u8) {
		for _ in 0..count {
			self.push(value);
		}
	}

	fn verbatim(&mut self, s: &[u8]) {
		self.extend_from_slice(s)
	}

	fn repeat(&mut self, count: usize, offset: usize) -> Result<()> {
		if !(1..=self.len()).contains(&offset) {
			return Err(Error::BadRepeat { count, offset, len: self.len() })
		}
		for _ in 0..count {
			self.push(self[self.len()-offset]);
		}
		Ok(())
	}
}

impl<'a> Output for OffsetVec<'a, u8> {
	fn constant(&mut self, count: usize, value: u8) {
		for _ in 0..count {
			self.push(value);
		}
	}

	fn verbatim(&mut self, s: &[u8]) {
		self.extend_from_slice(s)
	}

	fn repeat(&mut self, count: usize, offset: usize) -> Result<()> {
		if !(1..=self.len()).contains(&offset) {
			return Err(Error::BadRepeat { count, offset, len: self.len() })
		}
		for _ in 0..count {
			self.push(self[self.len()-offset]);
		}
		Ok(())
	}
}

pub struct CountSize(pub usize);

impl Output for CountSize {
	fn constant(&mut self, count: usize, _: u8) {
		self.0 += count;
	}

	fn verbatim(&mut self, s: &[u8]) {
		self.0 += s.len();
	}

	fn repeat(&mut self, count: usize, _: usize) -> Result<()> {
		self.0 += count;
		Ok(())
	}
}
