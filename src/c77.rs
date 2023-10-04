use gospel::read::{Reader, Le as _};

use crate::{Result, Error};
use crate::util::{OutBuf, Output};

pub fn decompress(f: &mut Reader, out: &mut Vec<u8>) -> Result<()> {
	let csize = f.u32()? as usize;
	let usize = f.u32()? as usize;
	let data = f.slice(csize)?;

	let start = out.len();
	decompress_inner(data, &mut OutBuf::new(out))?;
	Error::check_size(usize, out.len() - start)?;
	Ok(())
}

fn decompress_inner(data: &[u8], out: &mut impl Output) -> Result<()> {
	let f = &mut Reader::new(data);
	let mode = f.u32()?;
	if mode == 0 {
		out.verbatim(&data[4..]);
	} else {
		while !f.is_empty() {
			let x = f.u16()? as usize;
			let x1 = x & !(!0 << mode);
			let x2 = x >> mode;
			if x1 == 0 {
				out.verbatim(f.slice(x2)?);
			} else {
				out.repeat(x1, x2 + 1)?;
				out.verbatim(&[f.u8()?]);
			}
		}
	}
	Ok(())
}
