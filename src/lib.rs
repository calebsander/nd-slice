#![allow(incomplete_features)]
#![feature(array_zip)]
#![feature(generic_const_exprs)]
#![feature(new_uninit)]
#![feature(slice_ptr_get)]
#![feature(type_alias_impl_trait)]

mod ops;
mod util;
pub use ops::*;

use std::iter;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::slice;
use util::*;

// Newtype wrappers to clarify whether [usize; N] is being used
// as lengths, strides, or indices

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Len<const N: usize>([usize; N]);

#[derive(Clone, Copy)]
struct Stride<const N: usize>([usize; N]);

#[derive(Clone, Copy, Debug)]
struct Index<const N: usize>([usize; N]);

/// A range along a dimension, along with a number of indices to skip in between.
/// For example, "1.., selecting every 2nd element" would be represented as
/// Bounds { start: Some(1), end: None, step: 2 }
///
/// TODO: allow slicing in reverse
#[derive(Clone, Copy)]
pub struct Bounds {
  start: Option<usize>,
  end: Option<usize>,
  step: usize,
}

impl Bounds {
  pub fn all() -> Self {
    Self { start: None, end: None, step: 1 }
  }

  pub fn from(self, start: usize) -> Self {
    let Self { end, step, .. } = self;
    Self { start: Some(start), end, step }
  }

  pub fn to(self, end: usize) -> Self {
    let Self { start, step, .. } = self;
    Self { start, end: Some(end), step }
  }

  pub fn to_inclusive(self, end: usize) -> Self {
    self.to(end + 1)
  }

  pub fn step(self, step: usize) -> Self {
    let Self { start, end, .. } = self;
    Self { start, end, step }
  }
}

impl Default for Bounds {
  fn default() -> Self {
    Self::all()
  }
}

// Ideally we would have a custom pointer type ND<T, const N: usize>
// with metadata (Len<N>, Stride<N>). Then NDBox<T, N> would be Box<ND<T, N>>,
// NDSlice<'a, T, N> would be &'a ND<T, N>,
// and NDSliceMut<'a, T, N> would be &'a mut ND<T, N>.

/// The N-dimensional analog of Box<[T]>.
/// The underlying allocation is a Box<[T]> in row-major order.
/// Both the allocation length and the stride vector can be computed from `len`.
pub struct NDBox<T, const N: usize> {
  /// Pointer to the first element (index [0, ..., 0])
  data: NonNull<T>,
  /// Each dimension's number of indices. The indices along dimension D are 0..len[D].
  /// Each combination of dimension indices is an index into the N-dimensional slice.
  len: Len<N>,
}

/// N-dimensional analog of &[T]
pub struct NDSlice<'a, T, const N: usize> {
  data: NonNull<T>,
  len: Len<N>,
  /// The number of elements that need to be skipped in memory
  /// to advance by one in each direction
  stride: Stride<N>,
  // Pretend that we have a shared reference to a T with a lifetime of 'a.
  // This ensures the lifetime 'a is used, and enforces borrowing rules,
  // e.g. an NDSlice<'a, T, N> can't outlive the NDBox<T, N> it came from.
  phantom: PhantomData<&'a T>,
}

/// N-dimensional analog of &mut [T]
pub struct NDSliceMut<'a, T, const N: usize> {
  data: NonNull<T>,
  len: Len<N>,
  stride: Stride<N>,
  // Pretend that we have a mutable reference to a T with a lifetime of 'a.
  // This ensures the lifetime 'a is used, and enforces borrowing rules,
  // e.g. an NDSliceMut<'a, T, N> can't exist simultaneously with an NDSlice<'a, T, N>.
  phantom: PhantomData<&'a mut T>,
}

impl<const N: usize> Len<N> {
  /// Returns the number of elements in an N-dimensional slice with the given length
  fn size(self) -> usize {
    self.0.iter().product()
  }

  /// Returns the effective stride vector for an N-dimensional box's length.
  /// The strides are such that the elements are represented in row-major order.
  fn default_stride(self) -> Stride<N> {
    // Row-major order: indices are ordered by dimension 0, then 1, ..., N - 1.
    // So dimension N - 1 has stride 1, dimension N - 2 has stride len[N - 1], etc.
    let mut stride = Stride([0; N]);
    let mut next_stride = 1;
    for (dimension_stride, dimension_len) in iter::zip(&mut stride.0, self.0).rev() {
      *dimension_stride = next_stride;
      next_stride *= dimension_len;
    }
    stride
  }
}

/// Iterates over all indices from (0, ..., 0) up to `len`, repeating infinitely.
/// The indices are iterated in lexicographic order (the next index is (0, ..., 0, 1)),
/// matching the row-major order that NDBox stores its elements in.
struct IndexIterator<const N: usize> {
  len: Len<N>,
  index: Index<N>,
}

impl<const N: usize> IndexIterator<N> {
  fn new(len: Len<N>) -> Self {
    Self { len, index: Index([0; N]) }
  }
}

impl<const N: usize> Iterator for IndexIterator<N> {
  type Item = [usize; N];

  fn next(&mut self) -> Option<[usize; N]> {
    let old_index = self.index;
    let Self { index, len } = self;
    for (dimension_index, dimension_len) in iter::zip(&mut index.0, len.0).rev() {
      // Increment the index of the last dimension first
      *dimension_index += 1;
      if *dimension_index < dimension_len {
        break
      }

      // If we finish incrementing this dimension index,
      // reset it and increment the previous one
      *dimension_index = 0;
    }
    Some(old_index.0)
  }
}

impl<T, const N: usize> NDBox<T, N> {
  /// Creates an N-dimensional box with the given elements (in row-major order).
  /// SAFETY: `data` must have `len.size()` elements
  unsafe fn from_slice_unchecked(len: Len<N>, data: Box<[T]>) -> Self {
    debug_assert_eq!(data.len(), len.size());
    let data = NonNull::from(Box::leak(data)).as_non_null_ptr();
    Self { data, len }
  }

  /// Flattens a N-dimensional slice back into the boxed slice it came from
  fn to_box(self) -> Box<[T]> {
    let Self { data, len } = *ManuallyDrop::new(self);
    let data = data.as_ptr();
    let len = len.size();
    // SAFETY: this is the original allocation that was Box::leak()ed
    unsafe { Box::from_raw(slice::from_raw_parts_mut(data, len)) }
  }

  /// Creates a new NDBox of the specified length,
  /// initializing each element by calling the initializer with its index
  pub fn new_with<F: FnMut([usize; N]) -> T>(len: [usize; N], mut init: F) -> Self {
    let len = Len(len);
    let mut data = Box::new_uninit_slice(len.size());
    for (index, value) in IndexIterator::new(len).zip(&mut *data) {
      // TODO: panic safety: drop the initialized elements if this panics
      value.write(init(index));
    }
    // SAFETY: `data` has length `size(len)` and all elements were written to
    unsafe { Self::from_slice_unchecked(len, data.assume_init()) }
  }

  /// Creates a new NDBox of the specified length filled with the given value
  pub fn new_fill(len: [usize; N], value: T) -> Self where T: Clone {
    Self::new_with(len, |_| value.clone())
  }

  /// Creates a new NDBox of the specified length filled with the default value
  pub fn new_default(len: [usize; N]) -> Self where T: Default {
    Self::new_with(len, |_| T::default())
  }

  /// Creates a shared view of the data (like Deref for Box)
  pub fn as_slice(&self) -> NDSlice<T, N> {
    let Self { data, len } = *self;
    NDSlice { data, len, stride: self.len.default_stride(), phantom: PhantomData }
  }

  /// Creates a mutable view of the data (like DerefMut for Box)
  pub fn as_mut(&mut self) -> NDSliceMut<T, N> {
    let Self { data, len } = *self;
    NDSliceMut { data, len, stride: self.len.default_stride(), phantom: PhantomData }
  }

  /// Equivalent to NDSlice::get_unchecked()
  /// SAFETY: each dimension index must be less than the corresponding dimension length
  pub unsafe fn get_unchecked(&self, index: [usize; N]) -> &T {
    self.as_slice().get_unchecked(index)
  }

  /// Equivalent to NDSlice::get()
  pub fn get(&self, index: [usize; N]) -> Option<&T> {
    self.as_slice().get(index)
  }

  /// Equivalent to NDSliceMut::get_unchecked_mut()
  /// SAFETY: each dimension index must be less than the corresponding dimension length
  pub unsafe fn get_unchecked_mut(&mut self, index: [usize; N]) -> &mut T {
    self.as_mut().get_unchecked_mut(index)
  }

  /// Equivalent to NDSliceMut::get_mut()
  pub fn get_mut(&mut self, index: [usize; N]) -> Option<&mut T> {
    self.as_mut().get_mut(index)
  }

  /// Iterates over all elements by value, along with their index
  pub fn iter_owned(self) -> impl Iterator<Item = ([usize; N], T)> {
    IndexIterator::new(self.len).zip(self.to_box().into_vec())
  }
}

impl<T, const N: usize> Drop for NDBox<T, N> {
  fn drop(&mut self) {
    // Convert the N-dimensional slice back to its Box<[T]> allocation and drop it
    let Self { data, len } = *self;
    Self { data, len }.to_box();
  }
}

/// Like &[T], NDSlice<T, N> is copyable. So all methods take it by value.
impl<'a, T, const N: usize> NDSlice<'a, T, N> {
  /// Computes the location of the value at a given index in an N-dimensional slice.
  /// SAFETY: each dimension index must be at most the corresponding dimension length
  /// (so the resulting pointer does not go past the end of the underlying allocation)
  unsafe fn location(self, index: Index<N>) -> NonNull<T> {
    debug_assert!(
      iter::zip(index.0, self.len.0)
        .all(|(dimension_index, dimension_len)| dimension_index <= dimension_len),
    );
    let offset = iter::zip(index.0, self.stride.0)
      .map(|(dimension_index, dimension_stride)| dimension_index * dimension_stride)
      .sum();
    NonNull::new_unchecked(self.data.as_ptr().add(offset))
  }

  /// Returns whether an index is in bounds
  fn check_index(self, index: Index<N>) -> bool {
    iter::zip(index.0, self.len.0)
      .all(|(dimension_index, dimension_len)| dimension_index < dimension_len)
  }

  /// Iterates over the slices formed by extracting each index along the first dimension.
  /// (the slice cannot be 0-dimensional)
  fn rows(self) -> impl Iterator<Item = NDSlice<'a, T, {N - 1}>>
    where
      Is<{0 < N}>: True,
      [(); N - 1]: Sized, // redundant, but rustc can't figure this out
  {
    (0..self.len.0[0]).map(move |index0| self.extract::<0>(index0))
  }

  /// Accesses the element at the given index, without any bounds-checking.
  /// SAFETY: each dimension index must be less than the corresponding dimension length
  pub unsafe fn get_unchecked(self, index: [usize; N]) -> &'a T {
    let index = Index(index);
    debug_assert!(self.check_index(index));
    self.location(index).as_ref()
  }

  /// Accesses the element at the given index if it is in bounds, else returns None
  pub fn get(self, index: [usize; N]) -> Option<&'a T> {
    if !self.check_index(Index(index)) {
      return None
    }

    // SAFETY: index is in bounds
    Some(unsafe { self.get_unchecked(index) })
  }

  /// Like ops::Index but returns a reference with the slice lifetime 'a
  /// rather than the lifetime of the borrow (&self)
  pub fn index(self, index: [usize; N]) -> &'a T {
    let index = Index(index);
    assert!(self.check_index(index), "{:?} out of bounds for {:?}", index, self.len);
    // SAFETY: index is in bounds
    unsafe { self.get_unchecked(index.0) }
  }

  /// Picks out the elements at a given index along dimension `D`.
  /// The dimension is required to be a constant so it can be checked at compile time.
  pub fn extract<const D: usize>(self, dimension_index: usize) -> NDSlice<'a, T, {N - 1}>
    where Is<{D < N}>: True
  {
    let Self { len, stride, .. } = self;
    let dimension_len = len.0[D];
    assert!(
      dimension_index < dimension_len,
      "index {} out of bounds for dimension of len {}", dimension_index, dimension_len,
    );
    let mut index = Index([0; N]);
    index.0[D] = dimension_index;
    // SAFETY: index is in bounds
    let data = unsafe { self.location(index) };
    let len = Len(remove::<N, D>(len.0));
    let stride = Stride(remove::<N, D>(stride.0));
    NDSlice { data, len, stride, phantom: PhantomData }
  }

  /// Adds a new dimension at index `D` with the given length.
  /// Picking out any index along the new dimension will give the original slice.
  /// The dimension is required to be a constant so it can be checked at compile time.
  pub fn add_dimension<const D: usize>(self, dimension_len: usize)
    -> NDSlice<'a, T, {N + 1}>
    where Is<{D <= N}>: True
  {
    let Self { data, len, stride, .. } = self;
    let len = Len(insert::<N, D>(len.0, dimension_len));
    let stride = Stride(insert::<N, D>(stride.0, 0));
    NDSlice { data, len, stride, phantom: PhantomData }
  }

  /// Restricts the array to a slice along each dimension.
  /// Also allows applying an additional stride with Bounds::step().
  /// To leave a dimension unsliced, use Bounds::all() as its bounds.
  pub fn slice(self, bounds: [Bounds; N]) -> Self {
    let Self { len, stride, .. } = self;
    let dimensions = bounds.zip(len.0).zip(stride.0)
      .map(|((dimension_bounds, dimension_len), dimension_stride)| {
        let dimension_start = dimension_bounds.start.unwrap_or(0);
        let dimension_end = dimension_bounds.end.unwrap_or(dimension_len);
        let dimension_range = dimension_start..dimension_end;
        assert!(
          dimension_start <= dimension_end && dimension_end <= dimension_len,
          "range {:?} out of bounds for dimension of len {}",
          dimension_range, dimension_len,
        );
        let dimension_len = dimension_range.step_by(dimension_bounds.step).len();
        let dimension_stride = dimension_stride * dimension_bounds.step;
        (dimension_start, dimension_len, dimension_stride)
      });
    let index = Index(dimensions.map(|(dimension_start, _, _)| dimension_start));
    // SAFETY: `dimension_start`s have been checked to be in bounds
    let data = unsafe { self.location(index) };
    let len = Len(dimensions.map(|(_, dimension_len, _)| dimension_len));
    let stride = Stride(dimensions.map(|(_, _, dimension_stride)| dimension_stride));
    Self { data, len, stride, phantom: PhantomData }
  }

  /// Reverses the dimensions, so what was at index [a, ..., z] becomes index [z, ..., a].
  /// For a 2-dimensional slice, this is the matrix transpose operation.
  ///
  /// TODO: generalize this to allow any permutation of the dimensions
  pub fn transpose(mut self) -> Self {
    self.len.0.reverse();
    self.stride.0.reverse();
    self
  }

  /// Returns an iterator that will give each index in the slice
  pub fn indices(self) -> impl Iterator<Item = [usize; N]> {
    let len = self.len;
    // The iterator will repeat forever, so limit the number of elements
    IndexIterator::new(len).take(len.size())
  }

  /// Returns an iterator that will give each index in the slice along with its value
  pub fn iter(self) -> impl Iterator<Item = ([usize; N], &'a T)> {
    self.indices().map(move |index| (index, self.index(index)))
  }
}

/// Cloning an NDSlice is just copying the pointer, length, and stride.
/// (We don't #[derive(Clone, Copy)] because that unnecessarily requires T: Clone/Copy.)
impl<T, const N: usize> Clone for NDSlice<'_, T, N> {
  fn clone(&self) -> Self {
    *self
  }
}

impl<T, const N: usize> Copy for NDSlice<'_, T, N> {}

/// Like &mut [T], NDSlice<T, N> is NOT copyable.
/// All methods take it by reference so it can be re-borrowed.
impl<'a, T, const N: usize> NDSliceMut<'a, T, N> {
  /// Creates a shared view of the slice
  pub fn as_slice(&self) -> NDSlice<'a, T, N> {
    let Self { data, len, stride, .. } = *self;
    NDSlice { data, len, stride, phantom: PhantomData }
  }

  /// Equivalent to NDSlice::get_unchecked(), but mutably.
  /// SAFETY: each dimension index must be less than the corresponding dimension length
  pub unsafe fn get_unchecked_mut(&mut self, index: [usize; N]) -> &'a mut T {
    as_mut(self.as_slice().get_unchecked(index))
  }

  /// Equivalent to NDSlice::get(), but mutably
  pub fn get_mut(&mut self, index: [usize; N]) -> Option<&'a mut T> {
    // SAFETY: `self` has mutable access to all its values
    self.as_slice().get(index).map(|value| unsafe { as_mut(value) })
  }

  /// Like ops::IndexMut but returns a reference with the slice lifetime 'a
  /// rather than the lifetime of the borrow (&mut self)
  pub fn index_mut(&mut self, index: [usize; N]) -> &'a mut T {
    // SAFETY: `self` has mutable access to all its values
    unsafe { as_mut(self.as_slice().index(index)) }
  }

  /// Equivalent to NDSlice::extract(), but mutably
  pub fn extract_mut<const D: usize>(&mut self, dimension_index: usize)
    -> NDSliceMut<'a, T, {N - 1}>
    where Is<{D < N}>: True
  {
    let NDSlice { data, len, stride, .. } = self.as_slice().extract::<D>(dimension_index);
    NDSliceMut { data, len, stride, phantom: PhantomData }
  }

  /// Like NDSlice::add_dimension(), but mutably.
  /// Can only add a length of 1; otherwise, mutable references could alias.
  pub fn add_dimension_mut<const D: usize>(&mut self) -> NDSliceMut<'a, T, {N + 1}>
    where Is<{D <= N}>: True
  {
    let NDSlice { data, len, stride, .. } = self.as_slice().add_dimension::<D>(1);
    NDSliceMut { data, len, stride, phantom: PhantomData }
  }

  /// Equivalent to NDSlice::slice(), but mutably
  pub fn slice_mut(&mut self, bounds: [Bounds; N]) -> NDSliceMut<'a, T, N> {
    let NDSlice { data, len, stride, .. } = self.as_slice().slice(bounds);
    NDSliceMut { data, len, stride, phantom: PhantomData }
  }

  /// Equivalent to NDSlice::iter(), but mutably
  pub fn iter_mut(&mut self) -> impl Iterator<Item = ([usize; N], &mut T)> + '_ {
    self.as_slice().iter().map(|(index, value)| {
      // SAFETY: `self` has mutable access to all its values
      // and the iterator will return one mutable reference to each value,
      // so no mutable references will alias
      (index, unsafe { as_mut(value) })
    })
  }
}

/// A trait indicating that a slice's values can be iterated.
/// IntoIterator::Item reflects the ownership of the items,
/// e.g. NDBox<T, N>::Item is T, whereas NDSlice<'a, T, N>::Item is &'a T.
/// The indices are iterated in row-major order.
pub trait NDIntoIterator<const N: usize>: IntoIterator + Sized {
  /// Gets the N-dimensional length of the slice
  fn len(&self) -> [usize; N];

  /// Maps each value in a slice according to a function,
  /// producing a new boxed slice
  fn map<U, F: FnMut(Self::Item) -> U>(self, f: F) -> NDBox<U, N> {
    let len = Len(self.len());
    let data = self.into_iter().map(f).collect();
    unsafe { NDBox::from_slice_unchecked(len, data) }
  }

  /// Zips the corresponding values of two slices with the same length together
  fn zip<I: NDIntoIterator<N>>(self, other: I) -> iter::Zip<Self::IntoIter, I::IntoIter> {
    let len = Len(self.len());
    let other_len = Len(other.len());
    assert!(
      len == other_len,
      "Cannot operate on NDSlices with {:?} and {:?}", len, other_len,
    );
    iter::zip(self, other)
  }

  /// Zips the corresponding values of two slices with the same length together,
  /// mapping each pair of values according to a function to produce a new boxed slice
  fn zip_map<U, I, F>(self, other: I, mut f: F) -> NDBox<U, N>
    where
      I: NDIntoIterator<N>,
      F: FnMut(Self::Item, I::Item) -> U,
  {
    let len = Len(self.len());
    let data = self.zip(other).map(|(a, b)| f(a, b)).collect();
    // SAFETY: `data` has as many elements as `self` and `other`, which have length `len`
    unsafe { NDBox::from_slice_unchecked(len, data) }
  }
}

impl<T, const N: usize> IntoIterator for NDBox<T, N> {
  type Item = T;
  type IntoIter = impl Iterator<Item = T>;

  fn into_iter(self) -> Self::IntoIter {
    self.to_box().into_vec().into_iter()
  }
}

impl<T, const N: usize> NDIntoIterator<N> for NDBox<T, N> {
  fn len(&self) -> [usize; N] {
    self.len.0
  }
}

impl<'a, T, const N: usize> IntoIterator for NDSlice<'a, T, N> {
  type Item = &'a T;
  type IntoIter = impl Iterator<Item = &'a T>;

  fn into_iter(self) -> Self::IntoIter {
    self.iter().map(|(_, value)| value)
  }
}

impl<T, const N: usize> NDIntoIterator<N> for NDSlice<'_, T, N> {
  fn len(&self) -> [usize; N] {
    self.len.0
  }
}

impl<'a, T: 'a, const N: usize> IntoIterator for &'a mut NDSliceMut<'_, T, N> {
  type Item = &'a mut T;
  type IntoIter = impl Iterator<Item = &'a mut T> + 'a;

  fn into_iter(self) -> Self::IntoIter {
    self.iter_mut().map(|(_, value)| value)
  }
}

impl<T, const N: usize> NDIntoIterator<N> for &mut NDSliceMut<'_, T, N> {
  fn len(&self) -> [usize; N] {
    self.len.0
  }
}
