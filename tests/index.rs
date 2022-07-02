use nd_slice::{NDBox, NDIntoIterator};

mod util;
use util::*;

#[test]
fn test_basic() {
  let array = NDBox::from([
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
  ]);
  let array = array.as_slice();
  assert_eq!(array.len(), [3, 3]);
  assert_eq!(array[[0, 0]], 1);
  assert_eq!(array[[0, 1]], 2);
  assert_eq!(array[[0, 2]], 3);
  assert_eq!(array[[1, 0]], 4);
  assert_eq!(array[[1, 1]], 5);
  assert_eq!(array[[1, 2]], 6);
  assert_eq!(array[[2, 0]], 7);
  assert_eq!(array[[2, 1]], 8);
  assert_eq!(array[[2, 2]], 9);
  assert_eq!(format!("{:?}", array), "[[1, 2, 3], [4, 5, 6], [7, 8, 9]]");
}

#[test]
fn test_index_out_of_bounds() {
  let array = NDBox::from([
    [1, 2, 3, 4],
    [5, 6, 7, 8],
    [9, 10, 11, 12],
  ]);
  let array = array.as_slice();
  for row in 0..3 {
    for col in 0..4 {
      let expected_value = row * 4 + col + 1;
      assert_eq!(array[[row, col]], expected_value);
      assert_eq!(array.get([row, col]), Some(&expected_value));
    }
  }
  assert_panics_with(
    || drop(array[[3, 0]]),
    "Index([3, 0]) out of bounds for Len([3, 4])",
  );
  assert!(array.get([3, 0]).is_none());
  assert_panics_with(
    || drop(array[[0, 4]]),
    "Index([0, 4]) out of bounds for Len([3, 4])",
  );
  assert!(array.get([0, 4]).is_none());
  assert_panics_with(
    || drop(array[[3, 4]]),
    "Index([3, 4]) out of bounds for Len([3, 4])",
  );
  assert!(array.get([3, 4]).is_none());
  assert_panics_with(
    || drop(array[[usize::MAX, 0]]),
    &format!("Index([{}, 0]) out of bounds for Len([3, 4])", usize::MAX),
  );
  assert!(array.get([usize::MAX, 0]).is_none());
  assert_panics_with(
    || drop(array[[0, usize::MAX]]),
    &format!("Index([0, {}]) out of bounds for Len([3, 4])", usize::MAX),
  );
  assert!(array.get([0, usize::MAX]).is_none());
  assert_panics_with(
    || drop(array[[usize::MAX, usize::MAX]]),
    &format!("Index([{}, {}]) out of bounds for Len([3, 4])", usize::MAX, usize::MAX),
  );
  assert!(array.get([usize::MAX, usize::MAX]).is_none());
}

#[test]
fn test_mut() {
  let mut array = NDBox::new_fill([3, 4], 123.0);
  let mut array = array.as_mut();
  assert_eq!(
    array.as_slice(),
    NDBox::from([
      [123.0, 123.0, 123.0, 123.0],
      [123.0, 123.0, 123.0, 123.0],
      [123.0, 123.0, 123.0, 123.0],
    ]).as_slice(),
  );
  for index in array.as_slice().indices() {
    array[index] = (index[0] + 1) as f64 / (index[1] + 1) as f64;
  }
  assert_eq!(
    array.as_slice(),
    NDBox::from([
      [1.0 / 1.0, 1.0 / 2.0, 1.0 / 3.0, 1.0 / 4.0],
      [2.0 / 1.0, 2.0 / 2.0, 2.0 / 3.0, 2.0 / 4.0],
      [3.0 / 1.0, 3.0 / 2.0, 3.0 / 3.0, 3.0 / 4.0],
    ]).as_slice(),
  );
}

#[test]
fn test_indices() {
  let array = NDBox::new_fill([2, 3, 4], 0);
  let indices: Vec<_> = array.as_slice().indices().collect();
  assert_eq!(indices, [
    [0, 0, 0],
    [0, 0, 1],
    [0, 0, 2],
    [0, 0, 3],
    [0, 1, 0],
    [0, 1, 1],
    [0, 1, 2],
    [0, 1, 3],
    [0, 2, 0],
    [0, 2, 1],
    [0, 2, 2],
    [0, 2, 3],
    [1, 0, 0],
    [1, 0, 1],
    [1, 0, 2],
    [1, 0, 3],
    [1, 1, 0],
    [1, 1, 1],
    [1, 1, 2],
    [1, 1, 3],
    [1, 2, 0],
    [1, 2, 1],
    [1, 2, 2],
    [1, 2, 3],
  ]);
}

#[test]
fn test_indices_0_dimensions() {
  let array = NDBox::new_fill([], 0);
  let indices: Vec<_> = array.as_slice().indices().collect();
  assert_eq!(indices, [[]]);
}

#[test]
fn test_indices_length_0_last() {
  let array = NDBox::new_fill([3, 0], 0);
  let indices: Vec<_> = array.as_slice().indices().collect();
  let expected_indices: [[usize; 2]; 0] = [];
  assert_eq!(indices, expected_indices);
}

#[test]
fn test_indices_length_0_first() {
  let array = NDBox::new_fill([0, 3], 0);
  let indices: Vec<_> = array.as_slice().indices().collect();
  let expected_indices: [[usize; 2]; 0] = [];
  assert_eq!(indices, expected_indices);
}
