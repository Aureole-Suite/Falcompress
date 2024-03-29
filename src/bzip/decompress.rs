use gospel::read::{Le, Reader};

use crate::util::OutBuf;
use crate::{Error, Result};

struct Bits {
	bits: u16,
	// Zero's decompressor counts number of remaining bits instead,
	// but this method is simpler.
	nextbit: u16,
}

impl Bits {
	fn new() -> Self {
		Bits {
			bits: 0,
			nextbit: 0,
		}
	}

	fn bit(&mut self, f: &mut Reader) -> Result<bool> {
		if self.nextbit == 0 {
			self.renew_bits(f)?;
		}
		let v = self.bits & self.nextbit != 0;
		self.nextbit <<= 1;
		Ok(v)
	}

	fn renew_bits(&mut self, f: &mut Reader) -> Result<()> {
		self.bits = f.u16()?;
		self.nextbit = 1;
		Ok(())
	}

	fn bits(&mut self, n: usize, f: &mut Reader) -> Result<usize> {
		let mut x = 0;
		for _ in 0..n % 8 {
			x = x << 1 | usize::from(self.bit(f)?);
		}
		for _ in 0..n / 8 {
			x = x << 8 | f.u8()? as usize;
		}
		Ok(x)
	}

	fn read_count(&mut self, f: &mut Reader) -> Result<usize> {
		Ok(if self.bit(f)? {
			2
		} else if self.bit(f)? {
			3
		} else if self.bit(f)? {
			4
		} else if self.bit(f)? {
			5
		} else if self.bit(f)? {
			6 + self.bits(3, f)? //  6..=13
		} else {
			14 + self.bits(8, f)? // 14..=269
		})
	}
}

fn decompress_mode2(data: &[u8], mut w: OutBuf) -> Result<(), Error> {
	let f = &mut Reader::new(data);
	let mut b = Bits::new();
	b.renew_bits(f)?;
	b.nextbit <<= 8;

	loop {
		if !b.bit(f)? {
			w.extend(f.slice(1)?)
		} else if !b.bit(f)? {
			let o = b.bits(8, f)?;
			let n = b.read_count(f)?;
			w.decomp_repeat(n, o)?
		} else {
			match b.bits(13, f)? {
				0 => break,
				1 => {
					let n = if b.bit(f)? {
						b.bits(12, f)?
					} else {
						b.bits(4, f)?
					};
					w.decomp_constant(14 + n, f.u8()?);
				}
				o => {
					let n = b.read_count(f)?;
					w.decomp_repeat(n, o)?;
				}
			}
		}
	}
	Ok(())
}

#[bitmatch::bitmatch]
fn decompress_mode1(data: &[u8], mut w: OutBuf) -> Result<(), Error> {
	let f = &mut Reader::new(data);

	let mut last_o = 0;
	while !f.is_empty() {
		#[bitmatch]
		match f.u8()? as usize {
			"00xnnnnn" => {
				let n = if x == 1 { n << 8 | f.u8()? as usize } else { n };
				w.extend(f.slice(n)?);
			}
			"010xnnnn" => {
				let n = if x == 1 { n << 8 | f.u8()? as usize } else { n };
				w.decomp_constant(4 + n, f.u8()?);
			}
			"011nnnnn" => {
				w.decomp_repeat(n, last_o)?;
			}
			"1nnooooo" => {
				last_o = o << 8 | f.u8()? as usize;
				w.decomp_repeat(4 + n, last_o)?;
			}
		}
	}
	Ok(())
}

pub fn decompress(data: &[u8], w: &mut Vec<u8>) -> Result<()> {
	if data.first() == Some(&0) {
		decompress_mode2(data, w.into())
	} else {
		decompress_mode1(data, w.into())
	}
}
