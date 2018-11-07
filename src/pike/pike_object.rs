use ::pike::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{debug_master, Pike_interpreter_pointer, object, PIKE_T_OBJECT, schedule_really_free_object};
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


impl<TStorage> From<PikeObjectRef<TStorage>> for PikeThing {
    fn from(t: PikeObjectRef<TStorage>) -> Self {
        let untyped_obj: PikeObjectRef<()> = unsafe {::std::mem::transmute(t)};
        PikeThing::Object(untyped_obj)
    }
}

impl<TStorage> From<PikeObjectRef<TStorage>> for ::ffi::svalue {
    fn from(t: PikeObjectRef<TStorage>) -> Self {
        let a = ::ffi::anything { object: t.as_mut_ptr() };
        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_OBJECT as ::std::os::raw::c_ushort, subtype: 0 };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        ::ffi::svalue {u: a, tu: tu}
    }
}

impl<'ctx, TStorage> FromWithCtx<'ctx, PikeObjectRef<TStorage>>
for PikeObject<'ctx, TStorage> {
    fn from_with_ctx(obj_ref: PikeObjectRef<TStorage>, ctx: &'ctx PikeContext)
        -> Self {
        Self { object_ref: obj_ref, ctx: ctx, _phantom: PhantomData }
    }
}

/// Represents a Pike object.
/// Handles reference counting of the corresponding Pike object automatically.
/// The TStorage type represents the type of the object's storage.
#[derive(Debug)]
pub struct PikeObject<'ctx, TStorage>
where TStorage: Sized {
    object_ref: PikeObjectRef<TStorage>,
    ctx: &'ctx PikeContext,
    _phantom: PhantomData<TStorage>
}

impl<'ctx, TStorage> PikeObject<'ctx, TStorage> {
    /*
    pub unsafe fn from_ptr(object: *mut object, ctx: &'ctx PikeContext) -> Self {
        let obj_ref = PikeObjectRef::from_ptr(object);
        Self::from_with_ctx(obj_ref, ctx)
    }
    */

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
