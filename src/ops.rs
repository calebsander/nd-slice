use std::fmt::{self, Debug, Formatter};
use std::iter::Sum;
use std::ops::*;
use super::{NDBox, NDIntoIterator, NDSlice, NDSliceMut};

/// Clone each element in an NDBox, like Clone for Box<[T]>
impl<T: Clone, const N: usize> Clone for NDBox<T, N> {
  fn clone(&self) -> Self {
    self.as_slice().map(T::clone)
  }
}

/// A 0-dimensional slice is just a single value, so display it that way
impl<T: Debug> Debug for NDSlice<'_, T, 0> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self[[]].fmt(f)
  }
}

/// Display a 1-dimensional slice as a list
impl<T: Debug> Debug for NDSlice<'_, T, 1> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.rows()).finish()
  }
}

/// Display a 2-dimensional slice as a list of lists
impl<T: Debug> Debug for NDSlice<'_, T, 2> {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    f.debug_list().entries(self.rows()).finish()
  }
}

impl<T: Debug, const N: usize> Debug for NDBox<T, N>
  where for<'a> NDSlice<'a, T, N>: Debug
{
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.as_slice().fmt(f)
  }
}

impl<'a, T: Debug, const N: usize> Debug for NDSliceMut<'a, T, N>
  where NDSlice<'a, T, N>: Debug
{
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    self.as_slice().fmt(f)
  }
}

/// Two slices are equal iff their lengths and all corresponding values are equal
impl<T, U, const N: usize> PartialEq<NDSlice<'_, U, N>> for NDSlice<'_, T, N>
  where T: PartialEq<U>
{
  fn eq(&self, other: &NDSlice<U, N>) -> bool {
    self.len == other.len && self.zip(*other).all(|(lhs, rhs)| lhs == rhs)
  }
}

impl<T: Eq, const N: usize> Eq for NDSlice<'_, T, N> {}

impl<T: PartialEq<U>, U, const N: usize> PartialEq<NDBox<U, N>> for NDBox<T, N> {
  fn eq(&self, other: &NDBox<U, N>) -> bool {
    self.as_slice() == other.as_slice()
  }
}

impl<T: Eq, const N: usize> Eq for NDBox<T, N> {}

impl<T, U, const N: usize> PartialEq<NDSlice<'_, U, N>> for NDSliceMut<'_, T, N>
  where T: PartialEq<U>
{
  fn eq(&self, other: &NDSlice<U, N>) -> bool {
    self.as_slice() == *other
  }
}

impl<T, U, const N: usize> PartialEq<NDSliceMut<'_, U, N>> for NDSlice<'_, T, N>
  where T: PartialEq<U>
{
  fn eq(&self, other: &NDSliceMut<U, N>) -> bool {
    *self == other.as_slice()
  }
}

impl<T, U, const N: usize> PartialEq<NDSliceMut<'_, U, N>> for NDSliceMut<'_, T, N>
  where T: PartialEq<U>
{
  fn eq(&self, other: &NDSliceMut<U, N>) -> bool {
    self.as_slice() == other.as_slice()
  }
}

impl<T: Eq, const N: usize> Eq for NDSliceMut<'_, T, N> {}

/// 0-dimensional NDBox literal, e.g.:
/// NDBox::from(123)
impl<T> From<T> for NDBox<T, 0> {
  fn from(value: T) -> Self {
    let mut value = Some(value);
    Self::new_with([], |_| value.take().unwrap())
  }
}

/// 1-dimensional NDBox literal, e.g.:
/// NDBox::from([1, 2, 3])
impl<T, const L0: usize> From<[T; L0]> for NDBox<T, 1> {
  fn from(value: [T; L0]) -> Self {
    let mut value = value.map(Some);
    Self::new_with([L0], |[index0]| value[index0].take().unwrap())
  }
}

/// 2-dimensional NDBox literal, e.g.:
/// NDBox::from([
///   [1, 2, 3],
///   [4, 5, 6],
///   [7, 8, 9],
/// ])
///
/// Note that the input is a list of arrays, not slices,
/// which enforces at compile-time that their lengths are identical.
impl<T, const L0: usize, const L1: usize> From<[[T; L1]; L0]> for NDBox<T, 2> {
  fn from(value: [[T; L1]; L0]) -> Self {
    let mut value = value.map(|row_value| row_value.map(Some));
    Self::new_with([L0, L1], |[index0, index1]| value[index0][index1].take().unwrap())
  }
}

/// An index of an NDSlice can be dereferenced to the value at that index
impl<T, const N: usize> Index<[usize; N]> for NDSlice<'_, T, N> {
  type Output = T;

  fn index(&self, index: [usize; N]) -> &T {
    (*self).index(index)
  }
}

/// An index of an NDSliceMut can be dereferenced (mutably) to the value at that index
impl<T, const N: usize> Index<[usize; N]> for NDSliceMut<'_, T, N> {
  type Output = T;

  fn index(&self, index: [usize; N]) -> &T {
    self.as_slice().index(index)
  }
}

impl<T, const N: usize> IndexMut<[usize; N]> for NDSliceMut<'_, T, N> {
  fn index_mut(&mut self, index: [usize; N]) -> &mut T {
    self.index_mut(index)
  }
}

/// An index of an NDBox can be dereferenced (mutably) to the value at that index
impl<T, const N: usize> Index<[usize; N]> for NDBox<T, N> {
  type Output = T;

  fn index(&self, index: [usize; N]) -> &T {
    self.as_slice().index(index)
  }
}

impl<T, const N: usize> IndexMut<[usize; N]> for NDBox<T, N> {
  fn index_mut(&mut self, index: [usize; N]) -> &mut T {
    self.as_mut().index_mut(index)
  }
}

/// Perform an element-wise unary operation on a slice.
/// By using a generic NDIntoIterator, this can support negating:
/// - NDBox<T, N> (if T can be negated)
/// - NDSlice<T, N> (if &T can be negated)
/// (Can't implement Neg on I: NDIntoIterator<N> due to the orphan rule.)
macro_rules! arithmetic_unary_impl {
  ($trait:ident $func:ident) => {
    impl<T: $trait, const N: usize> $trait for NDBox<T, N> {
      type Output = NDBox<T::Output, N>;

      fn $func(self) -> Self::Output {
        self.map($trait::$func)
      }
    }

    impl<'a, T, const N: usize> $trait for NDSlice<'a, T, N> where &'a T: $trait {
      type Output = NDBox<<&'a T as $trait>::Output, N>;

      fn $func(self) -> Self::Output {
        self.map($trait::$func)
      }
    }
  };
}

arithmetic_unary_impl!{Neg neg}
arithmetic_unary_impl!{Not not}

/// Perform an element-wise binary operation on two slices with the same length.
/// By using a generic NDIntoIterator, this can support adding:
/// - NDBox<T, N> to NDBox<U, N> (if T can be added to U)
/// - NDSlice<T, N> to NDSlice<U, N> (if &T can be added to &U)
/// - NDBox<T, N> to NDSlice<U, N> (if T can be added to &U)
/// - NDSlice<T, N> to NDBox<U, N> (if &T can be added to U)
/// (Can't implement Add on I: NDIntoIterator<N> due to the orphan rule.)
macro_rules! arithmetic_binary_impl {
  ($trait:ident $func:ident) => {
    impl<T: $trait<U>, U, const N: usize> $trait<NDBox<U, N>> for NDBox<T, N> {
      type Output = NDBox<T::Output, N>;

      fn $func(self, rhs: NDBox<U, N>) -> Self::Output {
        self.zip_map(rhs, $trait::$func)
      }
    }

    impl<'a, 'b, T, U, const N: usize> $trait<NDSlice<'b, U, N>> for NDSlice<'a, T, N>
      where &'a T: $trait<&'b U>
    {
      type Output = NDBox<<&'a T as $trait<&'b U>>::Output, N>;

      fn $func(self, rhs: NDSlice<'b, U, N>) -> Self::Output {
        self.zip_map(rhs, $trait::$func)
      }
    }

    impl<'a, T, U, const N: usize> $trait<NDBox<U, N>> for NDSlice<'a, T, N>
      where &'a T: $trait<U>
    {
      type Output = NDBox<<&'a T as $trait<U>>::Output, N>;

      fn $func(self, rhs: NDBox<U, N>) -> Self::Output {
        self.zip_map(rhs, $trait::$func)
      }
    }

    impl<'a, T, U, const N: usize> $trait<NDSlice<'a, U, N>> for NDBox<T, N>
      where T: $trait<&'a U>
    {
      type Output = NDBox<T::Output, N>;

      fn $func(self, rhs: NDSlice<'a, U, N>) -> Self::Output {
        self.zip_map(rhs, $trait::$func)
      }
    }
  };
}

arithmetic_binary_impl!{Add add}
arithmetic_binary_impl!{BitAnd bitand}
arithmetic_binary_impl!{BitOr bitor}
arithmetic_binary_impl!{BitXor bitxor}
arithmetic_binary_impl!{Div div}
arithmetic_binary_impl!{Mul mul}
arithmetic_binary_impl!{Rem rem}
arithmetic_binary_impl!{Shl shl}
arithmetic_binary_impl!{Shr shr}
arithmetic_binary_impl!{Sub sub}

/// Perform an element-wise binary assignment on two slices with the same length.
/// By using a generic NDIntoIterator, this can support add-assigning:
/// - NDBox<T, N> to NDSliceMut<U, N> (if T can be added to &mut U)
/// - NDSlice<T, N> to NDSliceMut<U, N> (if &T can be added to &mut U)
macro_rules! arithmetic_assign_impl {
  ($trait:ident $op:ident) => {
    impl<'a, T, R, const N: usize> $trait<R> for NDSliceMut<'a, T, N>
      where
        R: NDIntoIterator<N>,
        T: $trait<R::Item>,
    {
      fn $op(&mut self, rhs: R) {
        for (lhs, rhs) in self.zip(rhs) {
          $trait::$op(lhs, rhs);
        }
      }
    }
  };
}

arithmetic_assign_impl!{AddAssign add_assign}
arithmetic_assign_impl!{BitAndAssign bitand_assign}
arithmetic_assign_impl!{BitOrAssign bitor_assign}
arithmetic_assign_impl!{BitXorAssign bitxor_assign}
arithmetic_assign_impl!{DivAssign div_assign}
arithmetic_assign_impl!{MulAssign mul_assign}
arithmetic_assign_impl!{RemAssign rem_assign}
arithmetic_assign_impl!{ShlAssign shl_assign}
arithmetic_assign_impl!{ShrAssign shr_assign}
arithmetic_assign_impl!{SubAssign sub_assign}

// Another example: matrix multiplication.
// A matrix (2-dimensional slice) with length [l0, l_inner] can be multiplied
// by another matrix with length [l_inner, l1], producing a matrix with length [l0, l1].
// Index [i0, i1] of the result matrix is the sum of the products of the
// corresponding elements of row l0 in the first matrix and column l1 in the second.

pub fn matrix_product<'a, 'b, T, U, O>(
  matrix1: NDSlice<'a, T, 2>,
  matrix2: NDSlice<'b, U, 2>,
) -> NDBox<O, 2> where
  &'a T: Mul<&'b U>,
  O: Sum<<&'a T as Mul<&'b U>>::Output>,
{
  let len1 = matrix1.len;
  let len2 = matrix2.len;
  let [length0, inner_length1] = len1.0;
  let [inner_length2, length1] = len2.0;
  assert!(
    inner_length1 == inner_length2,
    "Cannot multiply matrices of {:?} and {:?}", len1, len2,
  );
  NDBox::new_with([length0, length1], |[index0, index1]| {
    (0..inner_length1).map(|inner_index| {
      matrix1.index([index0, inner_index]) * matrix2.index([inner_index, index1])
    }).sum()
  })
}
