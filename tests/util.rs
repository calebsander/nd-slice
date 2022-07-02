use std::panic::{self, UnwindSafe};

pub fn assert_panics_with<F: FnOnce() + UnwindSafe>(f: F, expected_message: &str) {
  let actual_message = panic::catch_unwind(f).expect_err("Did not panic");
  let actual_message =
    // &'static str is used for panics with a constant message
    if let Some(&message) = actual_message.downcast_ref::<&str>() { message }
    // String is used for panics with formatted messages
    else if let Some(message) = actual_message.downcast_ref::<String>() { message }
    else { panic!("Did not panic with message") };
  assert_eq!(actual_message, expected_message, "Unexpected panic message");
}
