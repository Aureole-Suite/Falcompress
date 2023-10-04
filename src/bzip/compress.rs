mod mode1;
mod mode2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum CompressMode {
	Mode1,
	#[default]
	Mode2,
}

pub fn compress(input: &[u8], out: &mut Vec<u8>, mode: CompressMode) {
	match mode {
		CompressMode::Mode1 => mode1::compress(input, out),
		CompressMode::Mode2 => mode2::compress(input, out),
	}
}
