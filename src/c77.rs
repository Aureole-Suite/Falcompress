use gospel::read::{Reader, Le as _};

use crate::util::OutBuf;
use crate::{Result, Error};

pub fn decompress(f: &mut Reader, out: &mut Vec<u8>) -> Result<()> {
	let csize = f.u32()? as usize;
	let usize = f.u32()? as usize;
	let f = &mut Reader::new(f.slice(csize)?);

	let start = out.len();
	decompress_inner(f, out.into())?;
	Error::check_size(usize, out.len() - start)?;
	Error::check_end(f)?;
	Ok(())
}

fn decompress_inner(f: &mut Reader, mut out: OutBuf) -> Result<()> {
	let mode = f.u32()?;
	if mode == 0 {
		out.extend(f.slice(f.remaining().len())?);
	} else {
		while !f.is_empty() {
			let x = f.u16()? as usize;
			let x1 = x & !(!0 << mode);
			let x2 = x >> mode;
			if x1 == 0 {
				out.extend(f.slice(x2)?);
			} else {
				out.decomp_repeat(x1, x2 + 1)?;
				out.extend(&[f.u8()?]);
			}
		}
	}
	Ok(())
}
