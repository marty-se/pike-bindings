use ::pike::*;
use ::ffi::{debug_master, Pike_interpreter_pointer, object, PIKE_T_OBJECT};
use ::pike::interpreter::PikeContext;
use ::std::marker::PhantomData;

// Raw pointers (e.g. *mut array) are not Send-safe by default.
// However, we know that Pike won't free the array, leaving the pointer
// dangling, as long as we don't decrement the refcount we incremented in
// ::new().
unsafe impl<TStorage> Send for PikeObjectRef<TStorage> {}

#[derive(Debug)]
pub struct PikeObjectRef<TStorage> {
    object: *mut object,
    _phantom: PhantomData<TStorage>
}

impl<TStorage> PikeObjectRef<TStorage> {
    pub fn new(object: *mut object, _ctx: &PikeContext) -> Self {
        unsafe {
            (*object).refs += 1;
        }
        Self { object: object, _phantom: PhantomData }
    }

    pub fn new_without_ref(object: *mut object) -> Self {
        Self { object: object, _phantom: PhantomData }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.object, ctx)
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) ->
    PikeObject<'ctx, TStorage> {
        PikeObject { object_ref: self, ctx: ctx }
    }

    /// Returns the object of the current Pike execution context.
    pub fn current_object(ctx: &PikeContext) -> Self {
        let obj_ptr = unsafe {
            (*(*Pike_interpreter_pointer).frame_pointer).current_object
        };

        Self::new(obj_ptr, ctx)
    }

    /// Returns Pike's master object.
    pub fn get_master(ctx: &PikeContext) -> Self {
        Self::new(unsafe { debug_master() }, ctx)
    }

    pub fn as_mut_ptr(&self) -> *mut object {
        self.object
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
        let a = ::ffi::anything { object: t.object };
        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_OBJECT as ::std::os::raw::c_ushort, subtype: 0 };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        ::ffi::svalue {u: a, tu: tu}
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

impl<'ctx, TStorage> PikeObject<'ctx, TStorage> {
    pub fn from_ptr(object: *mut object, ctx: &'ctx PikeContext) -> Self {
        let obj_ref = PikeObjectRef::new(object, ctx);
        Self::from_ref(obj_ref, ctx)
    }

    pub fn from_ref(
        object_ref: PikeObjectRef<TStorage>,
        ctx: &'ctx PikeContext) -> Self {
        Self { object_ref: object_ref, ctx: ctx }
    }

    /// Returns the object of the current Pike execution context.
    pub fn current_object(ctx: &'ctx PikeContext) -> Self {
        Self::from_ref(PikeObjectRef::<TStorage>::current_object(ctx), ctx)
    }

    pub fn get_master(ctx: &'ctx PikeContext) -> Self {
        let master_ref = PikeObjectRef::<TStorage>::get_master(ctx);
        Self::from_ref(master_ref, ctx)
    }

    /// Calls a function in this Pike object.
    pub fn call_func(&self, func_name: &str, args: Vec<&PikeThing>)
    -> Result<PikeThing, PikeError> {
        let num_args = args.len() as i32;
        for a in args {
            self.ctx.push_to_stack(a.clone(self.ctx));
        }
        let func_cstr =
            ::std::ffi::CString::new(func_name).map_err(|e| e.to_string())?;
        self.ctx.catch_pike_error(|| {
            unsafe {
                ::ffi::apply(self.object_ref.object,
                    func_cstr.as_ptr(),
                    num_args);
            }
            self.ctx.pop_from_stack()
        })
    }

    /// Returns a reference to the data contained by this Pike object.
    pub fn wrapped<'s>(&'s self) -> &'s mut TStorage {
        unsafe {
            let ptr = (*self.object_ref.object).storage as *mut TStorage;
            &mut *ptr
        }
    }

    /// Replaces the storage of this object.
    pub fn update_data(&self, data: TStorage) {
        unsafe {
            let ptr = (*self.object_ref.object).storage as *mut TStorage;
            ::std::mem::drop(ptr);
            ::std::ptr::write(ptr, data);
        }
    }
}
