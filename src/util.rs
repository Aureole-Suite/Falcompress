use std::iter::zip;

use crate::{Result, Error};

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

pub(crate) struct OutBuf<'a> {
	start: usize,
	vec: &'a mut Vec<u8>,
}

impl<'a> From<&'a mut Vec<u8>> for OutBuf<'a> {
	fn from(vec: &'a mut Vec<u8>) -> Self {
		OutBuf { start: vec.len(), vec }
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
		if !(1..=self.len()-self.start).contains(&offset) {
			return Err(Error::BadRepeat { count, offset, len: self.len() })
		}
		for _ in 0..count {
			self.vec.push(self.vec[self.vec.len()-offset]);
		}
		Ok(())
	}
}
