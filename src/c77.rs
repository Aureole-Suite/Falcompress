use gospel::read::{Le as _, Reader};

use crate::util::{count_equal, OutBuf};
use crate::{Error, Result};

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

// Only supports mode 8, but that's the only one the game uses anyway so
pub fn compress_inner(input: &[u8], out: &mut Vec<u8>) {
	fn encode_raw(last: &mut usize, i: usize, out: &mut Vec<u8>, input: &[u8]) {
		while *last < i {
			let size = (i - *last).min(255);
			out.extend(&[0, size as u8]);
			out.extend(&input[*last..*last + size]);
			*last += size;
		}
	}
	let mut last = 0;
	let mut i = 0;

	while i < input.len() {
		if i - last == 255 {
			encode_raw(&mut last, i, out, input);
			continue;
		}

		let (start, len) = (i.saturating_sub(256)..i)
			.rev()
			.map(|j| (j, count_equal(&input[i..input.len() - 1], &input[j..], 255)))
			.max_by_key(|a| a.1)
			.unwrap_or((0, 0));

		let threshold = if i == last { 2 } else { 4 };
		if i - last < 252 && len >= threshold {
			encode_raw(&mut last, i, out, input);
			out.extend(&[len as u8, (i - start - 1) as u8, input[i + len]]);
			i += len + 1;
			last = i;
		} else {
			i += 1;
		}
	}
	encode_raw(&mut last, i, out, input);
}
