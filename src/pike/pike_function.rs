use ::pike::*;
use ::pike::interpreter::PikeContext;
use ::ffi::{svalue, apply_svalue, object};
use std::os::raw::c_ushort;

#[derive(Debug)]
pub struct PikeFunctionRef {
  pikeobj: PikeObjectRef<()>,
  fun_idx: c_ushort
}

impl PikeFunctionRef {
    pub fn new(object: *mut object, fun_idx: c_ushort, ctx: &PikeContext)
    -> Self {
        let pikeobj = PikeObjectRef::<()>::new(object, ctx);
        PikeFunctionRef { pikeobj: pikeobj, fun_idx: fun_idx }
    }

    pub fn new_without_ref(object: *mut object, fun_idx: c_ushort)
    -> Self {
        let pikeobj = PikeObjectRef::<()>::new_without_ref(object);
        PikeFunctionRef { pikeobj: pikeobj, fun_idx: fun_idx }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self { pikeobj: self.pikeobj.clone(ctx), fun_idx: self.fun_idx }
    }

    pub fn object_ptr(&self) -> *mut object {
        self.pikeobj.as_mut_ptr()
    }

    pub fn function_index(&self) -> u16 {
        self.fun_idx
    }
}

#[derive(Debug)]
pub struct PikeFunction<'ctx> {
  func_ref: PikeFunctionRef,
  ctx: &'ctx PikeContext
}

define_from_impls!(PikeFunctionRef, PikeFunction, Function, func_ref);

impl<'ctx> PikeFunction<'ctx> {
    pub fn new(object: *mut object, fun_idx: c_ushort, ctx: &'ctx PikeContext)
    -> Self {
        PikeFunction { func_ref: PikeFunctionRef::new(object, fun_idx, ctx), ctx }
    }

    pub fn call(&self, args: Vec<PikeThing>) -> Result<PikeThing, PikeError> {
        let num_args = args.len() as i32;
        for a in args {
            self.ctx.push_to_stack(a);
        }
        let mut func: svalue = self.into();
        self.ctx.catch_pike_error(|| {
            unsafe {
                apply_svalue(&mut func, num_args);
            }
            self.ctx.pop_from_stack()
        })
    }
}
