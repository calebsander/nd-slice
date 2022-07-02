use nd_slice::NDBox;

#[test]
fn test_transpose() {
  let array = NDBox::<_, 2>::from([
    [1, 2, 3],
    [4, 5, 6],
  ]);
  let array = array.as_slice();
  let transpose = array.transpose();
  assert_eq!(transpose, NDBox::from([
    [1, 4],
    [2, 5],
    [3, 6],
  ]).as_slice());
  assert_eq!(transpose.transpose(), array);
}
