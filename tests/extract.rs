use nd_slice::NDBox;

mod util;
use util::*;

fn array() -> NDBox<i32, 2> {
  NDBox::from([
    [1, -2, 3, -4],
    [-5, 6, -7, 8],
    [9, -10, 11, -12],
    [-13, 14, -15, 16],
  ])
}

#[test]
fn test_extract_0() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(array.extract::<0>(0), NDBox::from([1, -2, 3, -4]).as_slice());
  assert_eq!(array.extract::<0>(1), NDBox::from([-5, 6, -7, 8]).as_slice());
  assert_eq!(array.extract::<0>(2), NDBox::from([9, -10, 11, -12]).as_slice());
  assert_eq!(array.extract::<0>(3), NDBox::from([-13, 14, -15, 16]).as_slice());
}

#[test]
fn test_extract_1() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(array.extract::<1>(0), NDBox::from([1, -5, 9, -13]).as_slice());
  assert_eq!(array.extract::<1>(1), NDBox::from([-2, 6, -10, 14]).as_slice());
  assert_eq!(array.extract::<1>(2), NDBox::from([3, -7, 11, -15]).as_slice());
  assert_eq!(array.extract::<1>(3), NDBox::from([-4, 8, -12, 16]).as_slice());
}

#[test]
fn test_extract_twice() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(array.extract::<0>(0).extract::<0>(0), NDBox::from(1).as_slice());
  assert_eq!(array.extract::<0>(1).extract::<0>(2), NDBox::from(-7).as_slice());
  assert_eq!(array.extract::<1>(2).extract::<0>(1), NDBox::from(-7).as_slice());
  assert_eq!(array.extract::<0>(3).extract::<0>(2), NDBox::from(-15).as_slice());
  assert_eq!(array.extract::<1>(2).extract::<0>(3), NDBox::from(-15).as_slice());
}

#[test]
fn test_extract_out_of_bounds() {
  assert_panics_with(
    || drop(array().as_slice().extract::<0>(4)),
    "index 4 out of bounds for dimension of len 4",
  );
}
