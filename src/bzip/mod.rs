/// An implementation of Falcom's BZip compression algorithm,
/// used in *Trails in the Sky* as well as in their `itp` and `it3` file formats.
///
/// Note that this algorithm has no relation whatsoever to the bzip2 algorithm in common use.
///
/// BZip has two modes:
/// - Mode 1 appears to suffer less from barely-compressible data, but is only known to be supported by *Trails in the Sky*, which uses it in its 3d model files.
/// - Mode 2 is supported by all known games that use this algorithm, including *Trails in the Sky*.
///
/// There are also two framing modes. They have no known names, so I call them ed6 and ed7 from which game I first encountered them:
/// There is no known benefit for either of them, other than being in different contexts:
/// - *Trails in the Sky* uses `ed6` framing in its archive files.
/// - Certain forms of `itp` files also use ed6 framing. (Others use C77 compression, another proprietary Falcom algorithm.)
/// - `it3` files use ed7 framing.
///
/// Mode 2 is sometimes inofficially known as FALCOM2, and ed7 framing as FALCOM3.
use gospel::read::{Le as _, Reader};
use gospel::write::{Label, Le as _, Writer};

mod compress;
mod decompress;

use crate::{Error, Result};

/// Decompresses a single chunk of compressed data. Both mode 1 and 2 are supported.
/// There are no notable limitations regarding input or output size.
///
/// In most cases you will likely want to use the framed formats instead, [`decompress_ed6`] or [`decompress_ed7`].
pub use decompress::decompress as decompress_chunk;

pub fn decompress_ed6(f: &mut Reader) -> Result<Vec<u8>> {
	let mut out = Vec::new();
	loop {
		let Some(chunklen) = (f.u16()? as usize).checked_sub(2) else {
			return Err(Error::Frame);
		};
		decompress_chunk(f.slice(chunklen)?, &mut out)?;
		if f.u8()? == 0 {
			break;
		}
	}
	Ok(out)
}

pub fn decompress_ed7(f: &mut Reader) -> Result<Vec<u8>> {
	let csize = f.u32()? as usize;
	let f = &mut Reader::new(f.slice(csize)?);
	let usize = f.u32()? as usize;
	let nchunks = f.u32()? as usize;
	let mut out = Vec::with_capacity(usize);
	let mut chunk_lengths = Vec::with_capacity(nchunks);
	for n in 0..nchunks {
		let Some(chunklen) = (f.u16()? as usize).checked_sub(2) else {
			return Err(Error::Frame);
		};
		let start = out.len();
		decompress_chunk(f.slice(chunklen)?, &mut out)?;
		chunk_lengths.push(out.len() - start);

		// Falcom's tools always have 0/1 here, but some other tool — might even be one of mine — writes other values.
		if (f.u8()? != 0) != (n != nchunks - 1) {
			return Err(Error::Frame);
		}
	}

	// Falcom's tools always write a chunk of one extra byte. Other tools might not.
	// One notable exception is ao-psp's cti03200, which has *two* such bytes
	while out.len() > usize {
		if chunk_lengths.pop() != Some(1) {
			return Err(Error::Frame);
		}
		if out.pop().unwrap() != out[out.len() / 0x7FF0 * 0x7FF0] {
			return Err(Error::Frame);
		}
	}

	Error::check_size(usize, out.len())?;
	Error::check_end(f)?;
	Ok(out)
}

pub fn decompress_ed6_from_slice(data: &[u8]) -> Result<Vec<u8>> {
	decompress_ed6(&mut Reader::new(data))
}

pub fn decompress_ed7_from_slice(data: &[u8]) -> Result<Vec<u8>> {
	decompress_ed7(&mut Reader::new(data))
}

pub fn compression_info_ed6(data: &[u8]) -> Option<(usize, Option<CompressMode>)> {
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
		if f.u8().ok()? == 0 {
			if !f.remaining().is_empty() {
				return None;
			}
			let mut vec = Vec::new();
			decompress::decompress(chunk, &mut vec).ok()?;
			len += vec.len();
			break;
		} else {
			len += 0xFFF0;
		}
	}

	let mode = match (has_mode1, has_mode2) {
		(true, true) => None,
		(true, false) => Some(CompressMode::Mode1),
		(false, _) => Some(CompressMode::Mode2),
	};

	Some((len, mode))
}

/// Compresses a single chunk of compressed data, in the specified mode.
/// The mode 2 compressor can currently not handle chunks larger than `0xFFFF` bytes,
/// but mode 1 has no such restrictions.
/// Usually, chunks no larger than `0xFFF0` bytes are used, in either mode.
///
/// In most cases you will likely want to use the framed formats instead, [`compress_ed6`] or [`compress_ed7`].
pub use compress::compress as compress_chunk;
pub use compress::CompressMode;

pub fn compress_ed6(f: &mut Writer, data: &[u8], mode: CompressMode) {
	let mut nchunks = data.chunks(0xFFF0).count();
	for chunk in data.chunks(0xFFF0) {
		write_compressed_chunk(f, chunk, mode);
		nchunks -= 1;
		f.u8(nchunks as u8);
	}
}

pub fn compress_ed7(f: &mut Writer, data: &[u8], mode: CompressMode) {
	let start = Label::new();
	let end = Label::new();
	f.diff32(start, end);
	f.place(start);
	f.u32(data.len() as u32);
	f.u32(1 + data.chunks(0x7FF0).count() as u32);
	for chunk in data.chunks(0x7FF0) {
		write_compressed_chunk(f, chunk, mode);
		f.u8(1);
	}

	let dummy = *data
		.chunks(0x7FF0)
		.last()
		.and_then(|a| a.first())
		.unwrap_or(&0);
	write_compressed_chunk(f, &[dummy], mode);
	f.u8(0);

	f.place(end);
}

fn write_compressed_chunk(f: &mut Writer, chunk: &[u8], mode: CompressMode) {
	let start = f.here();
	let end = Label::new();
	f.diff16(start, end);
	let mut data = Vec::new();
	compress_chunk(chunk, &mut data, mode);
	f.slice(&data);
	f.place(end);
}

pub fn compress_ed6_to_vec(data: &[u8], mode: CompressMode) -> Vec<u8> {
	let mut w = Writer::new();
	compress_ed6(&mut w, data, mode);
	w.finish().unwrap()
}

pub fn compress_ed7_to_vec(data: &[u8], mode: CompressMode) -> Vec<u8> {
	let mut w = Writer::new();
	compress_ed7(&mut w, data, mode);
	w.finish().unwrap()
}

#[test]
#[ignore = "it is slow"]
fn mode2_should_roundtrip() {
	use gospel::read::{Le as _, Reader};

	let data = std::fs::read("../data/fc.extract2/00/font64._da").unwrap();
	let mut f = Reader::new(&data);
	let start = std::time::Instant::now();
	let mut d1 = std::time::Duration::ZERO;
	let mut d2 = std::time::Duration::ZERO;

	loop {
		let chunklen = f.u16().unwrap() as usize - 2;
		let inchunk = f.slice(chunklen).unwrap();
		assert!(inchunk[0] == 0);
		println!("{} / {}", f.pos(), f.len());

		let mut chunk = Vec::new();
		let start = std::time::Instant::now();
		decompress_chunk(inchunk, &mut chunk).unwrap();
		let end = std::time::Instant::now();
		d1 += end - start;

		let mut outchunk = Vec::new();
		let start = std::time::Instant::now();
		compress_chunk(&chunk, &mut outchunk, CompressMode::Mode2);
		let end = std::time::Instant::now();
		d2 += end - start;

		assert!(inchunk == outchunk);

		if f.u8().unwrap() == 0 {
			break;
		}
	}
	let end = std::time::Instant::now();

	println!(
		"Decompress {}, compress {}, total {}",
		d1.as_secs_f64(),
		d2.as_secs_f64(),
		(end - start).as_secs_f64()
	);
}

#[test]
fn mode1_should_roundtrip() {
	use gospel::read::{Le as _, Reader};

	let data = std::fs::read("../data/3rd.extract2/33/val2._x3").unwrap();
	let mut f = Reader::new(&data);
	let start = std::time::Instant::now();
	let mut d1 = std::time::Duration::ZERO;
	let mut d2 = std::time::Duration::ZERO;

	loop {
		let chunklen = f.u16().unwrap() as usize - 2;
		let inchunk = f.slice(chunklen).unwrap();
		assert!(inchunk[0] != 0);
		println!("{} / {}", f.pos(), f.len());

		let mut chunk = Vec::new();
		let start = std::time::Instant::now();
		decompress_chunk(inchunk, &mut chunk).unwrap();
		let end = std::time::Instant::now();
		d1 += end - start;

		let mut outchunk = Vec::new();
		let start = std::time::Instant::now();
		compress_chunk(&chunk, &mut outchunk, CompressMode::Mode1);
		let end = std::time::Instant::now();
		d2 += end - start;

		assert!(inchunk == outchunk);

		if f.u8().unwrap() == 0 {
			break;
		}
	}
	let end = std::time::Instant::now();

	println!(
		"Decompress {}, compress {}, total {}",
		d1.as_secs_f64(),
		d2.as_secs_f64(),
		(end - start).as_secs_f64()
	);
}
