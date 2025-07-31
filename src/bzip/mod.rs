//! An implementation of Falcom's BZip compression algorithm,
//! used in *Trails in the Sky* as well as in their `itp` and `it3` file formats.
//!
//! Note that this algorithm has no relation whatsoever to the bzip2 algorithm in common use.
//!
//! BZip has two modes:
//! - Mode 1 appears to suffer less from barely-compressible data, but is only known to be supported by *Trails in the Sky*, which uses it in its 3d model files.
//! - Mode 2 is supported by all known games that use this algorithm, including *Trails in the Sky*.
//!
//! There are also two framing modes. They have no known names, so I call them ed6 and ed7 from which game I first encountered them:
//! There is no known benefit for either of them, other than being in different contexts:
//! - *Trails in the Sky* uses `ed6` framing in its archive files.
//! - Certain forms of `itp` files also use ed6 framing. (Others use C77 compression, another proprietary Falcom algorithm.)
//! - `it3` files use ed7 framing.
//!
//! Mode 2 is sometimes inofficially known as FALCOM2, and ed7 framing as FALCOM3.
mod compress;
mod decompress;

/// Decompresses a single chunk of compressed data. Both mode 1 and 2 are supported.
/// There are no notable limitations regarding input or output size.
///
/// In most cases you will likely want to use the framed formats instead, [`crate::ed6`] or [`crate::ed7`].
pub use decompress::decompress;

/// Compresses a single chunk of compressed data, in the specified mode.
/// The mode 2 compressor can currently not handle chunks larger than `0xFFFF` bytes,
/// but mode 1 has no such restrictions.
/// Usually, chunks no larger than `0xFFF0` bytes are used, in either mode.
///
/// In most cases you will likely want to use the framed formats instead, [`crate::ed6`] or [`crate::ed7`].
pub use compress::compress;
pub use compress::CompressMode;

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
		decompress(inchunk, &mut chunk).unwrap();
		let end = std::time::Instant::now();
		d1 += end - start;

		let mut outchunk = Vec::new();
		let start = std::time::Instant::now();
		compress(&chunk, &mut outchunk, CompressMode::Mode2);
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
		decompress(inchunk, &mut chunk).unwrap();
		let end = std::time::Instant::now();
		d1 += end - start;

		let mut outchunk = Vec::new();
		let start = std::time::Instant::now();
		compress(&chunk, &mut outchunk, CompressMode::Mode1);
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
