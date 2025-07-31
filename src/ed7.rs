use gospel::read::{Le as _, Reader};
use gospel::write::{Label, Le as _, Writer};

use crate::{bzip, c77, Error, Result};
use crate::ed6::{read_compressed_chunk, run, write_compressed_chunk};

pub fn decompress(data: &[u8], out: &mut Vec<u8>) -> Result<usize> {
	let f = &mut Reader::new(data);
	let in_size = f.u32()? as usize;
	let expected_in_pos = f.pos() + in_size;
	let out_size = f.u32()? as usize;
	let expected_out_len = out.len() + out_size;
	let nchunks = f.u32()? as usize;
	for n in 0..nchunks {
		let chunk_len = read_compressed_chunk(f, out)?;

		if out.len() > expected_out_len {
			if chunk_len == 1 {
				// Falcom's tools always write a chunk of one extra byte.
				// In ao-psp cti03200, there's two.
				out.pop();
			} else {
				return Err(Error::Custom {
					message: format!("unexpected chunk at end of data: {chunk_len} bytes"),
				});
			}
		}

		// Falcom's tools always have 0/1 here, but some other tool — might even be one of mine — writes other values.
		let b = f.u8()?;
		if (b != 0) != (n != nchunks - 1) {
			return Err(Error::Custom {
				message: format!("unexpected byte at chunk {}/{}: {}", n, nchunks - 1, b),
			});
		}
	}

	Error::check_size("ed7 in_pos", expected_in_pos, f.pos())?;
	Error::check_size("ed7 out_pos", expected_out_len, out.len())?;
	Ok(f.pos())
}

pub fn freadp(data: &[u8], out: &mut Vec<u8>) -> Result<usize> {
	let f = &mut Reader::new(data);
	if f.check_u32(0x80000001).is_ok() {
		let n_chunks = f.u32()? as usize;
		let in_size = f.u32()? as usize;
		let buf_size = f.u32()? as usize;
		let out_size = f.u32()? as usize;
		let expected_in_pos = f.pos() + in_size;
		let expected_out_len = out.len() + out_size;

		let mut max_chunk_len = 0;
		for _ in 0..n_chunks {
			let chunk_len = run(f, |data| c77::decompress(data, out))?;
			max_chunk_len = max_chunk_len.max(chunk_len);
		}
		Error::check_size("freadp buf_size", buf_size, max_chunk_len)?;
		Error::check_size("freadp in_pos", expected_in_pos, f.pos())?;
		Error::check_size("freadp out_pos", expected_out_len, out.len())?;
	} else {
		run(f, |data| decompress(data, out))?;
	}
	Ok(f.pos())
}

pub fn compress(data: &[u8], mode: bzip::CompressMode) -> Vec<u8> {
	let mut f = Writer::new();
	let start = Label::new();
	let end = Label::new();
	let mut scratch = Vec::new();
	f.diff32(start, end);
	f.place(start);
	f.u32(data.len() as u32);
	f.u32(1 + data.chunks(0x7FF0).count() as u32);
	for chunk in data.chunks(0x7FF0) {
		write_compressed_chunk(&mut f, chunk, mode, &mut scratch);
		f.u8(1);
	}

	let dummy = *data
		.chunks(0x7FF0)
		.last()
		.and_then(|a| a.first())
		.unwrap_or(&0);
	write_compressed_chunk(&mut f, &[dummy], mode, &mut scratch);
	f.u8(0);

	f.place(end);
	f.finish().unwrap()
}
