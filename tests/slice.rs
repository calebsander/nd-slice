use nd_slice::{Bounds, NDBox};

mod util;
use util::*;

#[test]
fn test_step() {
  assert_eq!(
    NDBox::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]).as_slice()
      .slice([Bounds::all().step(3)]),
    NDBox::from([1, 4, 7, 10]).as_slice(),
  );
}

fn array() -> NDBox<i32, 2> {
  NDBox::from([
    [1, -2, 3, -4],
    [-5, 6, -7, 8],
    [9, -10, 11, -12],
    [-13, 14, -15, 16],
  ])
}

#[test]
fn test_slice_all() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(array.slice([Bounds::all(), Bounds::all()]), array);
}

#[test]
fn test_slice_both() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(
    array.slice([Bounds::all().from(1).to(3), Bounds::all().from(1).to(3)]),
    NDBox::from([
      [6, -7],
      [-10, 11],
    ]).as_slice(),
  );
}

#[test]
fn test_step_both() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(
    array.slice([Bounds::all().step(2), Bounds::all().step(2)]),
    NDBox::from([
      [1, 3],
      [9, 11],
    ]).as_slice(),
  );
}

#[test]
fn test_slice_and_step() {
  let array = array();
  let array = array.as_slice();
  assert_eq!(
    array.slice([Bounds::all().from(1).step(2), Bounds::all().from(2).step(2)]),
    NDBox::from([
      [-7],
      [-15],
    ]).as_slice(),
  );
}

#[test]
fn test_step_0() {
  let array = array();
  let array = array.as_slice();
  assert_panics_with(
    || drop(array.slice([Bounds::all().step(0), Bounds::all()])),
    "assertion failed: step != 0",
  );
}
