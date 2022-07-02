use nd_slice::NDBox;

#[test]
fn test_debug() {
  let array = NDBox::<_, 1>::from([1, 2, 3]);
  assert_eq!(format!("{:?}", array), "[1, 2, 3]");
  let array = NDBox::<_, 2>::from([
    ["A", "B", "C"],
    ["DE", "FG", "HI"],
    ["JKL", "MNO", "PQR"],
  ]);
  assert_eq!(
    format!("{:?}", array),
    r#"[["A", "B", "C"], ["DE", "FG", "HI"], ["JKL", "MNO", "PQR"]]"#,
  );
}

#[test]
fn test_equal() {
  assert_eq!(
    NDBox::<_, 2>::from([
      [1, 1],
      [1, 1],
    ]),
    NDBox::from([
      [1, 1],
      [1, 1],
    ]),
  );
}

#[test]
fn test_smaller_length1() {
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 1],
      [1, 1],
    ]),
    NDBox::from([
      [1],
      [1],
    ]),
  );
}

#[test]
fn test_larger_length1() {
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 1],
      [1, 1],
    ]),
    NDBox::from([
      [1, 1, 1],
      [1, 1, 1],
    ]),
  );
}

#[test]
fn test_smaller_length0() {
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 1],
      [1, 1],
    ]),
    NDBox::from([
      [1, 1],
    ]),
  );
}

#[test]
fn test_larger_length0() {
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 1],
      [1, 1],
    ]),
    NDBox::from([
      [1, 1],
      [1, 1],
      [1, 1],
    ]),
  );
}

#[test]
fn test_equal_0_dimensions() {
  assert_eq!(NDBox::from(1), NDBox::from(1));
}

#[test]
fn test_less_0_dimensions() {
  assert_ne!(NDBox::from(1), NDBox::from(2));
}

#[test]
fn test_greater_0_dimensions() {
  assert_ne!(NDBox::from(2), NDBox::from(1));
}

#[test]
fn test_equal_contents() {
  assert_eq!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
  );
}

#[test]
fn test_unequal_contents() {
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [7, 2, 3],
      [4, 5, 6],
    ]),
  );
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [1, 7, 3],
      [4, 5, 6],
    ]),
  );
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [1, 2, 7],
      [4, 5, 6],
    ]),
  );
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [7, 5, 6],
    ]),
  );
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 7, 6],
    ]),
  );
  assert_ne!(
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 6],
    ]),
    NDBox::<_, 2>::from([
      [1, 2, 3],
      [4, 5, 7],
    ]),
  );
}
