use ::pike::*;
use ::pike::error::catch_pike_error;
use ::std::marker::PhantomData;

/// Represents a Pike object.
/// Handles reference counting of the corresponding Pike object automatically.
/// The TStorage type represents the type of the object's storage.
#[derive(Debug)]
pub struct PikeObject<TStorage>
  where TStorage: Sized {
  object: *mut object,
  _phantom: PhantomData<TStorage>
}

impl<TStorage> PikeObject<TStorage> {
    /// Returns a new PikeObject instance
    pub fn new(object: *mut object) -> Self {
        unsafe {
            (*object).refs += 1;
        }
        PikeObject { object: object, _phantom: PhantomData }
    }

    /// Returns the object of the current Pike execution context.
    pub fn current_object() -> Self {
        unsafe {
            let obj_ptr = (*(*Pike_interpreter_pointer).frame_pointer).current_object;
            Self::new(obj_ptr)
        }
    }

    /// Calls a function in this Pike object.
    pub fn call_func(&self, func_name: &str, args: Vec<&PikeThing>)
    -> Result<PikeThing, PikeError> {
        for a in &args {
            a.push_to_stack();
        }
        let func_cstr = ::std::ffi::CString::new(func_name).map_err(|e| e.to_string())?;
        catch_pike_error(|| {
            unsafe {
                apply(self.object, func_cstr.as_ptr(), args.len() as i32);
            }
            PikeThing::pop_from_stack()
        })
    }

    pub fn as_mut_ptr(&self) -> *mut object {
        self.object
    }

    /// Returns a reference to the data contained by this Pike object.
    pub fn wrapped<'s>(&'s self) -> &'s mut TStorage {
        unsafe {
            let ptr = (*self.object).storage as *mut TStorage;
            &mut *ptr
        }
    }

    /// Replaces the storage of this object.
    pub fn update_data(&self, data: TStorage) {
        unsafe {
            let ptr = (*self.object).storage as *mut TStorage;
            ::std::mem::drop(ptr);
            ::std::ptr::write(ptr, data);
        }
    }

    /// Returns Pike's master object.
    pub fn get_master() -> Self {
        unsafe {
            Self::new(debug_master())
        }
    }
}

/// Converts a PikeObject to a generic PikeThing, discarding type information
/// about the PikeObject's TStorage type.
impl<TStorage> From<PikeObject<TStorage>> for PikeThing {
    fn from(t: PikeObject<TStorage>) -> Self {
        unsafe {
            PikeThing::Object(::std::mem::transmute(t))
        }
    }
}

impl<'a, TStorage> From<&'a PikeObject<TStorage>> for ::ffi::svalue {
    fn from(t: &PikeObject<TStorage>) -> Self {
        let a = ::ffi::anything { object: t.object };
        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_OBJECT as ::std::os::raw::c_ushort, subtype: 0 };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        return ::ffi::svalue {u: a, tu: tu};
    }
}

impl<TStorage> Clone for PikeObject<TStorage> {
    fn clone(&self) -> Self {
        unsafe {
            let object: *mut object = self.object;
            (*object).refs += 1;
        }
        PikeObject { object: self.object, _phantom: PhantomData }
    }
}

impl<TStorage> Drop for PikeObject<TStorage> {
    fn drop(&mut self) {
        unsafe {
            let object: *mut object = self.object;
            (*object).refs -= 1;
            if (*object).refs == 0 {
                schedule_really_free_object(object);
            }
        }
    }
}
