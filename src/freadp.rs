use gospel::read::{Le as _, Reader};

use crate::{Error, Result};

pub fn freadp_multi(f: &mut Reader, len: usize) -> Result<Vec<u8>> {
	let mut out = Vec::new();
	while out.len() < len {
		out.extend(freadp(f)?)
	}
	Ok(out)
}

pub fn freadp(f: &mut Reader) -> Result<Vec<u8>> {
	if f.check_u32(0x80000001).is_ok() {
		let n_chunks = f.u32()? as usize;
		let csize = f.u32()? as usize;
		let buf_size = f.u32()? as usize;
		let usize = f.u32()? as usize;
		let f = &mut Reader::new(f.slice(csize)?);

		let mut data = Vec::new();
		let mut max_csize = 0;
		for _ in 0..n_chunks {
			let start = f.pos();
			crate::c77::decompress(f, &mut data)?;
			max_csize = max_csize.max(f.pos() - start);
		}
		Error::check_size(buf_size, max_csize)?;
		Error::check_size(usize, data.len())?;
		Error::check_end(f)?;
		Ok(data)
	} else {
		Ok(crate::bzip::decompress_ed7(f)?)
	}
}
