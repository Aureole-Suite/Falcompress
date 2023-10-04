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

pub trait Output {
	fn constant(&mut self, n: usize, b: u8);
	fn verbatim(&mut self, s: &[u8]);
	fn repeat(&mut self, n: usize, o: usize) -> Result<()>;
}

pub struct OutBuf<'a> {
	start: usize,
	out: &'a mut Vec<u8>,
}

impl<'a> OutBuf<'a> {
	pub fn new(out: &'a mut Vec<u8>) -> Self {
		OutBuf {
			start: out.len(),
			out,
		}
	}
}

impl Output for OutBuf<'_> {
	fn constant(&mut self, n: usize, b: u8) {
		for _ in 0..n {
			self.out.push(b);
		}
	}

	fn verbatim(&mut self, s: &[u8]) {
		self.out.extend_from_slice(s)
	}

	fn repeat(&mut self, n: usize, o: usize) -> Result<()> {
		if !(1..=self.out.len()-self.start).contains(&o) {
			return Err(Error::BadRepeat { count: n, offset: o, len: self.out.len() })
		}
		for _ in 0..n {
			self.out.push(self.out[self.out.len()-o]);
		}
		Ok(())
	}
}

pub struct CountSize(pub usize);

impl Output for CountSize {
	fn constant(&mut self, n: usize, _: u8) {
		self.0 += n;
	}

	fn verbatim(&mut self, s: &[u8]) {
		self.0 += s.len();
	}

	fn repeat(&mut self, n: usize, _: usize) -> Result<()> {
		self.0 += n;
		Ok(())
	}
}
