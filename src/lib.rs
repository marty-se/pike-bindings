#![feature(unboxed_closures, fn_traits)]
#![feature(proc_macro)]
extern crate pike_macros;

extern crate serde;

#[macro_use]
extern crate quick_error;

pub mod pike;
pub mod bindings;

pub mod module {
  pub use pike_macros::init_pike_module as init_pike_module;
  pub use pike_macros::pike_func_inits as pike_func_inits;
  pub use pike_macros::pike_export as pike_export;
  pub use pike::PikeError as PikeError;

  pub use pike::error::prepare_error_message as prepare_error_message;
  pub use pike::error::pike_error as pike_error;
}