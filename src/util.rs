// Helper type to express constraints on constants in where clauses.
// Since `True` is only implemented for `Is<true>`,
// the constraint `Is<X>: True` requires `X` to be true.

use std::ptr::NonNull;

pub enum Is<const B: bool> {}

pub trait True {}

impl True for Is<true> {}

/// An unsafe function to turn a reference into a mutable one
pub unsafe fn as_mut<T>(value: &T) -> &mut T {
  NonNull::from(value).as_mut()
}

/// Insert a value at index `I` of `input`
pub fn insert<const N: usize, const I: usize>(input: [usize; N], value: usize)
  -> [usize; N + 1]
  where Is<{I <= N}>: True
{
  let mut result = [value; N + 1];
  result[..I].copy_from_slice(&input[..I]);
  result[I + 1..].copy_from_slice(&input[I..]);
  result
}

/// Remove the value at index `I` of `input`
pub fn remove<const N: usize, const I: usize>(input: [usize; N]) -> [usize; N - 1]
  where Is<{I < N}>: True
{
  let mut result = [0; N - 1];
  result[..I].copy_from_slice(&input[..I]);
  result[I..].copy_from_slice(&input[I + 1..]);
  result
}
