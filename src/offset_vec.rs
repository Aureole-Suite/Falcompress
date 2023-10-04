#![allow(clippy::missing_safety_doc)]

use std::collections::TryReserveError;
use std::fmt::{self, Formatter, Debug};
use std::io::{self, IoSlice};
use std::mem::MaybeUninit;
use std::ops::{Deref, DerefMut, RangeBounds, Bound, Index, IndexMut};
use std::vec::{Drain, Splice};

pub struct OffsetVec<'a, T> {
	offset: usize,
	vec: &'a mut Vec<T>,
}

impl<'a, T> OffsetVec<'a, T> {
	pub fn new(vec: &'a mut Vec<T>) -> Self {
		OffsetVec {
			offset: vec.len(),
			vec,
		}
	}

	pub fn delimit(&mut self) -> OffsetVec<T> {
		OffsetVec::new(self.vec)
	}

	pub fn capacity(&self) -> usize {
		self.vec.capacity() - self.offset
	}
	pub fn reserve(&mut self, additional: usize) {
		self.vec.reserve(additional)
	}
	pub fn reserve_exact(&mut self, additional: usize) {
		self.vec.reserve_exact(additional)
	}
	pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.vec.try_reserve(additional)
	}
	pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
		self.vec.try_reserve_exact(additional)
	}
	pub fn shrink_to_fit(&mut self) {
		self.vec.shrink_to_fit()
	}
	pub fn shrink_to(&mut self, min_capacity: usize) {
		self.vec.shrink_to(min_capacity + self.offset)
	}
	pub fn truncate(&mut self, len: usize) {
		self.vec.truncate(len + self.offset)
	}
	pub fn as_slice(&self) -> &[T] {
		&self.vec[self.offset..]
	}
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		&mut self.vec[self.offset..]
	}
	pub fn as_ptr(&self) -> *const T {
		unsafe { self.vec.as_ptr().add(self.offset) }
	}
	pub fn as_mut_ptr(&mut self) -> *mut T {
		unsafe { self.vec.as_mut_ptr().add(self.offset) }
	}
	pub unsafe fn set_len(&mut self, new_len: usize) {
		self.vec.set_len(new_len + self.offset)
	}
	pub fn swap_remove(&mut self, index: usize) -> T {
		self.vec.swap_remove(index + self.offset)
	}
	pub fn insert(&mut self, index: usize, element: T) {
		self.vec.insert(index + self.offset, element)
	}
	pub fn remove(&mut self, index: usize) -> T {
		self.vec.remove(index + self.offset)
	}
	// pub fn retain<F>(&mut self, f: F) where F: FnMut(&T) -> bool;
	// pub fn retain_mut<F>(&mut self, f: F) where F: FnMut(&mut T) -> bool;
	// pub fn dedup_by_key<F, K>(&mut self, key: F) where F: FnMut(&mut T) -> K, K: PartialEq<K>;
	// pub fn dedup_by<F>(&mut self, same_bucket: F) where F: FnMut(&mut T, &mut T) -> bool;
	// pub fn dedup(&mut self) where T: PartialEq;
	pub fn push(&mut self, value: T) {
		self.vec.push(value)
	}
	pub fn pop(&mut self) -> Option<T> {
		if self.is_empty() {
			None
		} else {
			self.vec.pop()
		}
	}
	pub fn append(&mut self, other: &mut Vec<T>) {
		self.vec.append(other)
	}
	pub fn drain<R>(&mut self, range: R) -> Drain<'_, T> where
		R: RangeBounds<usize>,
	{
		self.vec.drain(self.map_range(range))
	}
	pub fn clear(&mut self) {
		self.truncate(0)
	}
	pub fn len(&self) -> usize {
		self.vec.len() - self.offset
	}
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}
	pub fn split_off(&mut self, at: usize) -> Vec<T> {
		self.vec.split_off(at + self.offset)
	}
	pub fn resize_with<F>(&mut self, new_len: usize, f: F) where F: FnMut() -> T {
		self.vec.resize_with(new_len + self.offset, f)
	}
	pub fn spare_capacity_mut(&mut self) -> &mut [MaybeUninit<T>] {
		self.vec.spare_capacity_mut()
	}
	pub fn splice<R, I>(&mut self, range: R, replace_with: I)
		-> Splice<'_, <I as IntoIterator>::IntoIter> where
		R: RangeBounds<usize>,
		I: IntoIterator<Item = T>
	{
		self.vec.splice(self.map_range(range), replace_with)
	}
	pub fn resize(&mut self, new_len: usize, value: T) where T: Clone {
		self.vec.resize(new_len + self.offset, value)
	}
	pub fn extend_from_slice(&mut self, other: &[T]) where T: Clone {
		self.vec.extend_from_slice(other)
	}
	pub fn extend_from_within<R>(&mut self, src: R) where
		T: Clone,
		R: RangeBounds<usize>,
	{
		self.vec.extend_from_within(self.map_range(src))
	}

	fn map_range<R>(&self, range: R) -> (Bound<usize>, Bound<usize>) where R: RangeBounds<usize> {
		fn map_bound(offset: usize, v: Bound<&usize>) -> Bound<usize> {
			match v {
				Bound::Included(v) => Bound::Included(*v + offset),
				Bound::Excluded(v) => Bound::Excluded(*v + offset),
				Bound::Unbounded => Bound::Unbounded,
			}
		}
		(
			map_bound(self.offset, range.start_bound()),
			map_bound(self.offset, range.end_bound()),
		)
	}
}

impl<'a, T: Debug> Debug for OffsetVec<'a, T> {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		self.as_slice().fmt(f)
	}
}

impl<'a, T> Deref for OffsetVec<'a, T> {
	type Target = [T];

	fn deref(&self) -> &[T] {
		&self.vec[self.offset..]
	}
}

impl<'a, T> DerefMut for OffsetVec<'a, T> {
	fn deref_mut(&mut self) -> &mut [T] {
		&mut self.vec[self.offset..]
	}
}

impl<'a, T, U> Extend<U> for OffsetVec<'a, T> where Vec<T>: Extend<U> {
	fn extend<I: IntoIterator<Item = U>>(&mut self, iter: I) {
		self.vec.extend(iter)
	}
}

impl<'a, T, I> Index<I> for OffsetVec<'a, T> where [T]: Index<I> {
	type Output = <[T] as Index<I>>::Output;

	fn index(&self, index: I) -> &Self::Output {
		self.as_slice().index(index)
	}
}

impl<'a, T, I> IndexMut<I> for OffsetVec<'a, T> where [T]: IndexMut<I> {
	fn index_mut(&mut self, index: I) -> &mut Self::Output {
		self.as_mut_slice().index_mut(index)
	}
}

impl<'a, 'b, T> IntoIterator for &'b OffsetVec<'a, T> {
	type Item = &'b T;
	type IntoIter = std::slice::Iter<'b, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, 'b, T> IntoIterator for &'b mut OffsetVec<'a, T> {
	type Item = &'b mut T;
	type IntoIter = std::slice::IterMut<'b, T>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

impl<'a, T, E> PartialEq<E> for OffsetVec<'a, T> where Vec<T>: PartialEq<E> {
	fn eq(&self, other: &E) -> bool {
		self.vec == other
	}
}

impl<'a> io::Write for OffsetVec<'a, u8> {
	#[inline]
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.extend_from_slice(buf);
		Ok(buf.len())
	}

	#[inline]
	fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
		let len = bufs.iter().map(|b| b.len()).sum();
		self.reserve(len);
		for buf in bufs {
			self.extend_from_slice(buf);
		}
		Ok(len)
	}

	#[inline]
	fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
		self.extend_from_slice(buf);
		Ok(())
	}

	#[inline]
	fn flush(&mut self) -> io::Result<()> {
		Ok(())
	}
}
