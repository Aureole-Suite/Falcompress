use std::iter::zip;

pub fn count_equal(a: &[u8], b: &[u8], limit: usize) -> usize {
	let n = limit.min(a.len()).min(b.len());
	const N: usize = 8;

	let mut i = 0;
	for (a, b) in zip(a[..n].chunks_exact(N), b[..n].chunks_exact(N)) {
		if a == b {
			i += N;
		} else {
			let a = u64::from_le_bytes(a.try_into().unwrap());
			let b = u64::from_le_bytes(b.try_into().unwrap());
			return i + ((a ^ b).trailing_zeros() / 8) as usize;
		}
	}

	i = n.saturating_sub(N);
	zip(&a[i..n], &b[i..n])
		.take_while(|(a, b)| a == b)
		.count() + i
}
