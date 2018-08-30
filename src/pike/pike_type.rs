use ::ffi::{pike_type, PIKE_T_TYPE, really_free_pike_type};

#[derive(Debug)]
pub struct PikeType {
  pike_type: *mut pike_type
}

def_pike_type!(PikeType, pike_type, type_, PIKE_T_TYPE, really_free_pike_type);
