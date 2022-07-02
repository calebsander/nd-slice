#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use nd_slice::*;

fn main() {
  // High temperatures over 10 days in 3 cities
  let temperatures_fahrenheit = NDBox::<f32, 2>::from([
    // NYC, LAX, CHI
    [72.0, 80.0, 79.0], // 2022-06-01
    [79.0, 79.0, 79.0], // 2022-06-02
    [76.0, 73.0, 83.0], // 2022-06-03
    [80.0, 70.0, 72.0], // 2022-06-04
    [77.0, 75.0, 81.0], // 2022-06-05
    [80.0, 77.0, 76.0], // 2022-06-06
    [78.0, 76.0, 71.0], // 2022-06-07
    [82.0, 75.0, 72.0], // 2022-06-08
    [81.0, 80.0, 80.0], // 2022-06-09
    [77.0, 81.0, 82.0], // 2022-06-10
  ]);
  dbg!(temperatures_fahrenheit.as_slice());
  let [days, cities] = temperatures_fahrenheit.len();
  dbg!(days, cities);
  let const_32 = NDBox::from(32.0);
  let const_32 = const_32.as_slice()
    .add_dimension::<0>(days)
    .add_dimension::<1>(cities);
  dbg!(const_32);
  let const_1_8 = NDBox::from(1.8);
  let const_1_8 = const_1_8.as_slice()
    .add_dimension::<0>(days)
    .add_dimension::<1>(cities);
  let temperatures_celsius =
    (temperatures_fahrenheit.as_slice() - const_32) / const_1_8;
  let temperatures_celsius = temperatures_celsius.as_slice();
  dbg!(temperatures_celsius);
  let average_temperatures = NDBox::new_with([cities], |[city]| {
    let city_temperatures = temperatures_celsius.extract::<1>(city);
    city_temperatures.into_iter().sum::<f32>() / days as f32
  });
  let average_temperatures = average_temperatures.as_slice();
  dbg!(average_temperatures);
}
