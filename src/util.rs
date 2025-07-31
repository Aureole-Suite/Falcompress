use std::iter::zip;

use crate::{Error, Result};

pub(crate) fn count_equal(a: &[u8], b: &[u8], limit: usize) -> usize {
	let n = limit.min(a.len()).min(b.len());
	const N: usize = 8;

	let mut i = 0;
	let a = a[..n].as_chunks::<N>();
	let b = b[..n].as_chunks::<N>();
	for (a, b) in zip(a.0, b.0) {
		if a == b {
			i += N;
		} else {
			let a = u64::from_le_bytes(*a);
			let b = u64::from_le_bytes(*b);
			return i + ((a ^ b).trailing_zeros() / 8) as usize;
		}
	}

	i + zip(a.1, b.1).take_while(|(a, b)| a == b).count()
}

pub(crate) struct OutBuf<'a> {
	start: usize,
	vec: &'a mut Vec<u8>,
}

impl<'a> From<&'a mut Vec<u8>> for OutBuf<'a> {
	fn from(vec: &'a mut Vec<u8>) -> Self {
		OutBuf {
			start: vec.len(),
			vec,
		}
	}
}

impl<'a> std::ops::Deref for OutBuf<'a> {
	type Target = Vec<u8>;

	fn deref(&self) -> &Self::Target {
		self.vec
	}
}

impl<'a> std::ops::DerefMut for OutBuf<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.vec
	}
}

impl OutBuf<'_> {
	pub(crate) fn decomp_constant(&mut self, count: usize, value: u8) {
		for _ in 0..count {
			self.push(value);
		}
	}

	pub(crate) fn decomp_repeat(&mut self, count: usize, offset: usize) -> Result<()> {
		if !(1..=self.len() - self.start).contains(&offset) {
			return Err(Error::BadRepeat {
				count,
				offset,
				len: self.len(),
			});
		}
		for _ in 0..count {
			self.vec.push(self.vec[self.vec.len() - offset]);
		}
		Ok(())
	}
}
