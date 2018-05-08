#![feature(proc_macro)]
extern crate pike_macros;

extern crate serde;

pub mod pike;
pub mod bindings;

pub mod module {
  pub use pike_macros::init_pike_module as init_pike_module;
  pub use pike_macros::pike_export as pike_export;
}