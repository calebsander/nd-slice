use nd_slice::NDBox;

fn array() -> NDBox<i32, 1> {
  NDBox::from([1, 2, 3])
}

#[test]
fn test_row_vector() {
  assert_eq!(array().as_slice().add_dimension::<0>(1), NDBox::from([
    [1, 2, 3],
  ]).as_slice());
}

#[test]
fn test_col_vector() {
  assert_eq!(array().as_slice().add_dimension::<1>(1), NDBox::from([
    [1],
    [2],
    [3],
  ]).as_slice());
}

#[test]
fn test_0_rows() {
  assert_eq!(
    array().as_slice().add_dimension::<0>(0),
    NDBox::new_fill([0, 3], 0).as_slice(),
  );
}

#[test]
fn test_0_cols() {
  assert_eq!(
    array().as_slice().add_dimension::<1>(0),
    NDBox::new_fill([3, 0], 0).as_slice(),
  );
}

#[test]
fn test_3_rows() {
  assert_eq!(array().as_slice().add_dimension::<0>(3), NDBox::from([
    [1, 2, 3],
    [1, 2, 3],
    [1, 2, 3],
  ]).as_slice());
}

#[test]
fn test_3_cols() {
  assert_eq!(array().as_slice().add_dimension::<1>(3), NDBox::from([
    [1, 1, 1],
    [2, 2, 2],
    [3, 3, 3],
  ]).as_slice());
}
