use ::pike::*;
use ::ffi::{pike_type};

#[derive(Debug)]
pub struct PikeTypeRef {
  pike_type: *mut pike_type
}

impl PikeTypeRef {
    pub fn new(pike_type: *mut pike_type, _ctx: &PikeContext) -> Self {
        unsafe {
            (*pike_type).refs += 1;
        }
        Self { pike_type: pike_type }
    }

    pub fn new_without_ref(pike_type: *mut pike_type) -> Self {
        Self { pike_type: pike_type }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.pike_type, ctx)
    }

    pub fn as_mut_ptr(&self) -> *mut pike_type {
        self.pike_type
    }
}