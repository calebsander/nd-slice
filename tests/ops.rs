use nd_slice::{matrix_product, Bounds, NDBox};

mod util;
use util::*;

#[test]
fn test_new_with() {
  let array = NDBox::new_with([6, 3], |[i, j]| i as i32 - j as i32);
  assert_eq!(array, NDBox::from([
    [0, -1, -2],
    [1, 0, -1],
    [2, 1, 0],
    [3, 2, 1],
    [4, 3, 2],
    [5, 4, 3],
  ]));
}

fn left() -> NDBox<i32, 2> {
  NDBox::from([
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
  ])
}

fn right() -> NDBox<i32, 2> {
  NDBox::from([
    [123, 456, 789],
    [234, 567, 891],
    [345, 678, 912],
  ])
}

#[test]
fn test_neg_values() {
  assert_eq!(-left(), NDBox::from([
    [-1, -2, -3],
    [-4, -5, -6],
    [-7, -8, -9],
  ]));
}

#[test]
fn test_neg_ref() {
  assert_eq!(-right().as_slice(), NDBox::from([
    [-123, -456, -789],
    [-234, -567, -891],
    [-345, -678, -912],
  ]));
}

#[test]
fn test_add_values() {
  assert_eq!(left() + right(), NDBox::from([
    [124, 458, 792],
    [238, 572, 897],
    [352, 686, 921],
  ]));
}

#[test]
fn test_add_ref() {
  let left = left();
  let left = left.as_slice();
  let right = right();
  let right = right.as_slice();
  assert_eq!(
    left.extract::<0>(1) + right.extract::<1>(1),
    NDBox::from([460, 572, 684]),
  );
}

#[test]
fn test_mismatched_lengths() {
  let left = left();
  let right = right();
  let right = right.as_slice();
  assert_panics_with(
    || drop(left + right.slice([Bounds::all().to(2), Bounds::all()])),
    "Cannot operate on NDSlices with Len([3, 3]) and Len([2, 3])",
  );
}

#[test]
fn test_add_assign_values() {
  let mut slice = NDBox::<_, 2>::from([
    [1, 2, 3],
    [4, 5, 6],
  ]);
  let mut slice = slice.as_mut();
  let add_values = NDBox::from([
    [10, 11, 12],
    [-10, -11, -12],
  ]);
  slice += add_values;
  assert_eq!(slice, NDBox::from([
    [11, 13, 15],
    [-6, -6, -6],
  ]).as_slice());
}


#[test]
fn test_add_assign_ref() {
  let mut slice = NDBox::<_, 2>::from([
    [1, 2, 3],
    [4, 5, 6],
  ]);
  let mut slice = slice.as_mut();
  let add_values = NDBox::from([
    [10, 11, 12],
    [-10, -11, -12],
  ]);
  slice += add_values.as_slice();
  assert_eq!(slice, NDBox::from([
    [11, 13, 15],
    [-6, -6, -6],
  ]).as_slice());
}

fn matrix() -> NDBox<i32, 2> {
  NDBox::from([
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9],
    [10, 11, 12],
  ])
}

#[test]
fn test_matrix_product() {
  let matrix = matrix();
  let matrix = matrix.as_slice();
  assert_eq!(
    matrix_product::<_, _, i32>(matrix.transpose(), matrix),
    NDBox::from([
      [
        1 * 1 + 4 * 4 + 7 * 7 + 10 * 10,
        1 * 2 + 4 * 5 + 7 * 8 + 10 * 11,
        1 * 3 + 4 * 6 + 7 * 9 + 10 * 12,
      ],
      [
        2 * 1 + 5 * 4 + 8 * 7 + 11 * 10,
        2 * 2 + 5 * 5 + 8 * 8 + 11 * 11,
        2 * 3 + 5 * 6 + 8 * 9 + 11 * 12,
      ],
      [
        3 * 1 + 6 * 4 + 9 * 7 + 12 * 10,
        3 * 2 + 6 * 5 + 9 * 8 + 12 * 11,
        3 * 3 + 6 * 6 + 9 * 9 + 12 * 12,
      ],
    ]),
  );
  assert_eq!(
    matrix_product::<_, _, i32>(matrix, matrix.transpose()),
    NDBox::from([
      [
        1 * 1 + 2 * 2 + 3 * 3,
        1 * 4 + 2 * 5 + 3 * 6,
        1 * 7 + 2 * 8 + 3 * 9,
        1 * 10 + 2 * 11 + 3 * 12,
      ],
      [
        4 * 1 + 5 * 2 + 6 * 3,
        4 * 4 + 5 * 5 + 6 * 6,
        4 * 7 + 5 * 8 + 6 * 9,
        4 * 10 + 5 * 11 + 6 * 12,
      ],
      [
        7 * 1 + 8 * 2 + 9 * 3,
        7 * 4 + 8 * 5 + 9 * 6,
        7 * 7 + 8 * 8 + 9 * 9,
        7 * 10 + 8 * 11 + 9 * 12,
      ],
      [
        10 * 1 + 11 * 2 + 12 * 3,
        10 * 4 + 11 * 5 + 12 * 6,
        10 * 7 + 11 * 8 + 12 * 9,
        10 * 10 + 11 * 11 + 12 * 12,
      ],
    ]),
  );
}

fn identity(len: usize) -> NDBox<i32, 2> {
  NDBox::new_with([len, len], |[i, j]| (i == j) as i32)
}

#[test]
fn test_matrix_product_identity() {
  let matrix = matrix();
  let matrix = matrix.as_slice();
  let identity_3 = identity(3);
  let identity_3 = identity_3.as_slice();
  let identity_4 = identity(4);
  let identity_4 = identity_4.as_slice();
  assert_eq!(matrix_product::<_, _, i32>(matrix, identity_3).as_slice(), matrix);
  assert_eq!(matrix_product::<_, _, i32>(identity_4, matrix).as_slice(), matrix);
}

#[test]
fn test_matrix_product_invalid_lengths() {
  let matrix = matrix();
  let matrix = matrix.as_slice();
  assert_panics_with(|| {
    let _: NDBox<i32, 2> = matrix_product(matrix, matrix);
  }, "Cannot multiply matrices of Len([4, 3]) and Len([4, 3])");

  let identity_3 = identity(3);
  let identity_3 = identity_3.as_slice();
  let identity_4 = identity(4);
  let identity_4 = identity_4.as_slice();
  assert_panics_with(|| {
    let _: NDBox<i32, 2> = matrix_product(identity_3, identity_4);
  }, "Cannot multiply matrices of Len([3, 3]) and Len([4, 4])");
}
