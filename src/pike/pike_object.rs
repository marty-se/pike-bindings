use ::pike::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{debug_master, Pike_interpreter_pointer, object,
    schedule_really_free_object, svalue};
use ::pike::interpreter::PikeContext;
use ::std::marker::PhantomData;

#[derive(Debug)]
pub struct PikeObjectRef<TStorage> {
    ptr: *mut object,
    _phantom: PhantomData<TStorage>
}

refcounted_type_with_storage!(PikeObjectRef, object, DeferredObjectDrop);

struct DeferredObjectDrop {
    ptr: *mut object
}

impl DropWithContext for DeferredObjectDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                schedule_really_free_object(ptr);
            }
        }
    }
}

impl<TStorage> PikeObjectRef<TStorage> {
    /// Returns the object of the current Pike execution context.
    pub fn current_object(ctx: &PikeContext) -> Self {
        unsafe {
            let obj_ptr =
                (*(*Pike_interpreter_pointer).frame_pointer).current_object;
            Self::from_ptr_add_ref(obj_ptr, ctx)
        }
    }

    /// Returns Pike's master object.
    pub fn get_master(ctx: &PikeContext) -> Self {
        unsafe { Self::from_ptr_add_ref(debug_master(), ctx) }
    }
}

/// Represents a Pike object.
/// Handles reference counting of the corresponding Pike object automatically.
/// The TStorage type represents the type of the object's storage.
#[derive(Debug)]
pub struct PikeObject<'ctx, TStorage>
where TStorage: Sized {
    object_ref: PikeObjectRef<TStorage>,
    ctx: &'ctx PikeContext
}

define_from_impls_with_storage!(PikeObjectRef, PikeObject, Object, object_ref);

impl<'ctx, TStorage> PikeObject<'ctx, TStorage> {
    /// Returns the object of the current Pike execution context.
    pub fn current_object(ctx: &'ctx PikeContext) -> Self {
        Self::from_with_ctx(PikeObjectRef::<TStorage>::current_object(ctx), ctx)
    }

    pub fn get_master(ctx: &'ctx PikeContext) -> Self {
        let master_ref = PikeObjectRef::<TStorage>::get_master(ctx);
        Self::from_with_ctx(master_ref, ctx)
    }

    /// Calls a function in this Pike object.
    pub fn call_func(&self, func_name: &str, args: Vec<&PikeThing>)
    -> Result<PikeThing, PikeError> {
        let num_args = args.len() as i32;
        for a in args {
            self.ctx.push_to_stack(a.clone_with_ctx(self.ctx));
        }
        let func_cstr =
            ::std::ffi::CString::new(func_name).map_err(|e| e.to_string())?;
        self.ctx.catch_pike_error(|| {
            unsafe {
                ::ffi::apply(self.object_ref.as_mut_ptr(),
                    func_cstr.as_ptr(),
                    num_args);
            }
            self.ctx.pop_from_stack()
        })
    }

    /// Returns a reference to the data contained by this Pike object.
    pub fn wrapped<'s>(&'s self) -> &'s mut TStorage {
        unsafe {
            let ptr = (*self.object_ref.as_mut_ptr()).storage as *mut TStorage;
            &mut *ptr
        }
    }

    /// Replaces the storage of this object.
    pub fn update_data(&self, data: TStorage) {
        unsafe {
            let ptr = (*self.object_ref.as_mut_ptr()).storage as *mut TStorage;
            ::std::mem::drop(ptr);
            ::std::ptr::write(ptr, data);
        }
    }
}
