//! Rust bindings to the Pike interpreter.
//!
//! # Pike's Interpreter Lock
//! Pike's interpreter lock must be held while accessing or modifying Pike's
//! data structures. The safe Rust API in this crate ensures that the
//! interpreter lock is acquired for applicable operations by requiring a
//! reference to a `PikeContext`.
//!
//! References to Pike things (arrays, mappings, objects etc.) can be held
//! without an active `PikeContext`. These are represented by `PikeArrayRef`,
//! `PikeMappingRef` etc. Such references can be converted into the first-class
//! representation like this:
//! ```
//! let array_ref: PikeArrayRef = ...;
//! PikeContext::call_with_context(|ctx| {
//!   let array: PikeArray = array_ref.into_with_ctx(ctx);
//! });
//! ```
//!
//! Rust's usual lifetime constraints will make sure that the first-class
//! representations cannot outlive the `PikeContext` instance, so accessing
//! cannot take place without holding the interpreter lock.
extern crate pike_macros;
extern crate serde;
extern crate lazy_static;

mod ffi;
pub mod interpreter;

#[macro_use]
mod macros;

pub mod traits;
pub mod types;

pub mod module {
  pub use pike_macros::init_pike_module as init_pike_module;
  pub use pike_macros::pike_func_inits as pike_func_inits;
  pub use pike_macros::pike_export as pike_export;
  pub use interpreter::PikeError as PikeError;

  pub use interpreter::prepare_error_message as prepare_error_message;

  pub use interpreter::{PikeContext};
}
