use ::pike::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{pike_type, really_free_pike_type};

#[derive(Debug)]
pub struct PikeTypeRef {
  ptr: *mut pike_type
}

refcounted_type!(PikeTypeRef, pike_type, DeferredTypeDrop);

struct DeferredTypeDrop {
    ptr: *mut pike_type
}

impl DropWithContext for DeferredTypeDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                really_free_pike_type(ptr);
            }
        }
    }
}
