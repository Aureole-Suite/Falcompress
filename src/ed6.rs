use gospel::read::{Le as _, Reader};
use gospel::write::{Le as _, Writer};

use crate::{bzip, Error, Result};

pub fn decompress(data: &[u8], out: &mut Vec<u8>) -> Result<usize> {
	let mut f = Reader::new(data);
	loop {
		read_compressed_chunk(&mut f, out)?;
		if f.u8()? == 0 {
			break;
		}
	}
	Ok(f.pos())
}

pub fn inspect(data: &[u8]) -> Option<(usize, Option<bzip::CompressMode>)> {
	let f = &mut Reader::new(data);
	let mut len = 0;
	let mut has_mode1 = false;
	let mut has_mode2 = false;
	loop {
		let chunklen = (f.u16().ok()? as usize).checked_sub(2)?;
		let chunk = f.slice(chunklen).ok()?;
		if chunk.is_empty() {
			return None;
		}
		if chunk[0] == 0 {
			has_mode2 = true;
		} else {
			has_mode1 = true
		};
		if f.u8().ok()? != 0 {
			len += 0xFFF0;
		} else {
			if !f.remaining().is_empty() {
				return None;
			}
			// We have to decompress the last chunk to get its length
			let mut vec = Vec::new();
			bzip::decompress(chunk, &mut vec).ok()?;
			len += vec.len();
			break;
		}
	}

	let mode = match (has_mode1, has_mode2) {
		(true, true) => None,
		(true, false) => Some(bzip::CompressMode::Mode1),
		(false, _) => Some(bzip::CompressMode::Mode2),
	};

	Some((len, mode))
}

pub fn compress(data: &[u8], mode: bzip::CompressMode) -> Vec<u8> {
	let mut f = Writer::new();
	let mut nchunks = data.chunks(0xFFF0).count();
	let mut scratch = Vec::new();
	for chunk in data.chunks(0xFFF0) {
		write_compressed_chunk(&mut f, chunk, mode, &mut scratch);
		nchunks -= 1;
		f.u8(nchunks as u8);
	}
	f.finish().unwrap()
}

pub(crate) fn run(f: &mut Reader, mut func: impl FnMut(&[u8]) -> Result<usize>) -> Result<usize> {
	let len = func(f.remaining())?;
	f.slice(len)?;
	Ok(len)
}

pub(crate) fn read_compressed_chunk(f: &mut Reader, out: &mut Vec<u8>) -> Result<usize> {
	let start = out.len();
	let expected_chunk_end = f.pos() + f.u16()? as usize;
	run(f, |data| bzip::decompress(data, out))?;
	Error::check_size(expected_chunk_end, f.pos())?;
	Ok(out.len() - start)
}

pub(crate) fn write_compressed_chunk(f: &mut Writer, chunk: &[u8], mode: bzip::CompressMode, scratch: &mut Vec<u8>) {
	scratch.clear();
	bzip::compress(chunk, scratch, mode);
	f.u16(scratch.len() as u16 + 2);
	f.slice(scratch);
}
