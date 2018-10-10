//use ::ffi::*;
use std::ptr;

#[macro_use]
mod macros;

mod pike_array;
pub use self::pike_array::{PikeArray, PikeArrayRef};

mod pike_function;
pub use self::pike_function::{PikeFunction, PikeFunctionRef};

mod pike_float;
pub use self::pike_float::PikeFloat;

mod pike_int;
pub use self::pike_int::PikeInt;

mod pike_mapping;
pub use self::pike_mapping::{PikeMapping, PikeMappingRef};

mod pike_multiset;
pub use self::pike_multiset::{PikeMultiset, PikeMultisetRef};

mod pike_object;
pub use self::pike_object::{PikeObject, PikeObjectRef};

mod pike_program;
pub use self::pike_program::{PikeProgram, PikeProgramRef};

mod pike_str;
pub use self::pike_str::{PikeString, PikeStringRef};

mod pike_type;
pub use self::pike_type::{PikeTypeRef};

mod pike_svalue;
mod pike_thing;
pub use self::pike_thing::*;

pub mod error;

pub use self::error::PikeError;

pub mod interpreter;
pub use self::interpreter::PikeContext;
