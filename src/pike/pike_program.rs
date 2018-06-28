use ::bindings::*;
use ::pike::{PikeObject, PikeThing, PikeError};
use ::pike::error::catch_pike_error;
use std::ffi::CString;
use ::std::marker::PhantomData;

pub use ::bindings::{low_add_storage, pike_set_prog_event_callback, PROG_EVENT_INIT, PROG_EVENT_EXIT};

#[derive(Debug)]
pub struct PikeProgram<TStorage>
  where TStorage: Sized {
  program: *mut program,
  _phantom: PhantomData<TStorage>
}

impl<TStorage> PikeProgram<TStorage> {
    pub fn new(program: *mut program) -> Self {
        unsafe {
            (*program).refs += 1;
        }
        PikeProgram { program: program, _phantom: PhantomData }
    }

    pub fn clone_object(&self) -> Result<PikeObject<()>, PikeError> {
        catch_pike_error(|| {
            unsafe {
                debug_clone_object(self.program, 0);
            }

            let pt = PikeThing::pop_from_stack();
            if let PikeThing::Object(obj) = pt {
                return obj;
            }
            panic!("clone_object didn't return an object");
        })
    }

    // FIXME: Should return a Result
    pub fn clone_object_with_data(&self, data: TStorage)
      -> Result<PikeObject<TStorage>, PikeError> {
          catch_pike_error(|| {
              unsafe {
                  debug_clone_object(self.program, 0);
              }

              let pt = PikeThing::pop_from_stack();
              if let PikeThing::Object(obj) = pt {
                  // We know that the cloned object's TStorage type
                  // is equivalent to that of the program, but since
                  // we can't pass that type information through the C API
                  // we have to transmute.
                  let res: PikeObject<TStorage>;
                  unsafe { res = ::std::mem::transmute(obj); }
                  {
                      let storage: &mut TStorage = res.wrapped();
                      *storage = data;
                  }
                  return res;
              }
              panic!("clone_object didn't return an object");
          })
    }

    pub fn index(&self, index: &str) -> PikeThing {
        let mut index_prog_sval: svalue = self.into();
        let index_val_thing: PikeThing = index.into();
        let mut index_val: svalue = (&index_val_thing).into();
        let mut res: svalue = Default::default();
        unsafe {
            program_index_no_free(&mut res, &mut index_prog_sval,
                &mut index_val);
        }
        (&res).into()
    }

    /// Returns the program that is currently being compiled.
    pub fn current_compilation() -> Self {
        unsafe {
            PikeProgram {
                program: (*Pike_compiler).new_program,
                _phantom: PhantomData
            }
        }
    }
}

impl<'a, TStorage> From<&'a PikeProgram<TStorage>> for ::bindings::svalue {
    fn from(t: &PikeProgram<TStorage>) -> Self {
        let a = ::bindings::anything { program: t.program };
        let t = ::bindings::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_OBJECT as ::std::os::raw::c_ushort, subtype: 0 };
        let tu = ::bindings::svalue__bindgen_ty_1 {t: t};
        return ::bindings::svalue {u: a, tu: tu};
    }
}

impl<TStorage> Clone for PikeProgram<TStorage> {
    fn clone(&self) -> Self {
        unsafe {
            let program: *mut program = self.program;
            (*program).refs += 1;
        }
        PikeProgram { program: self.program, _phantom: PhantomData }
    }
}

impl<TStorage> Drop for PikeProgram<TStorage> {
    fn drop(&mut self) {
        unsafe {
            let program: *mut program = self.program;
            (*program).refs -= 1;
            if (*program).refs == 0 {
                really_free_program(program);
            }
        }
    }
}

unsafe extern "C" fn prog_event_callback<TStorage>(event: i32)
  where TStorage: Default {
    match event as u32 {
      PROG_EVENT_INIT => {
        let storage_data: TStorage = Default::default();
        let storage_ptr = (*(*Pike_interpreter_pointer).frame_pointer).current_storage
          as *mut TStorage;
        ::std::ptr::write(storage_ptr, storage_data);
      },
      PROG_EVENT_EXIT => {
          let storage = (*(*Pike_interpreter_pointer).frame_pointer).current_storage
            as *mut TStorage;
          ::std::mem::drop(storage);
      },
      _ => {}
    }
}

pub fn start_new_program<TStorage>(filename: &str, line: u32)
  where TStorage: Default {
  unsafe {
    let fname = ::std::ffi::CString::new(filename).unwrap();
    debug_start_new_program(line as i64, fname.as_ptr());
    low_add_storage(::std::mem::size_of::<TStorage>(), ::std::mem::align_of::<TStorage>(), 0);
    pike_set_prog_event_callback(Some(prog_event_callback::<TStorage>));
  }
}

pub fn end_class(name: &str) {
  let class_name = ::std::ffi::CString::new(name).unwrap();
  unsafe {
    let prog: *mut program = debug_end_program();
    add_program_constant(class_name.as_ptr(), prog, 0);
  }
}

pub fn set_prog_event_callback(fun: unsafe extern "C" fn(i32) -> ()) {
  unsafe {
    pike_set_prog_event_callback(Some(fun));
  }
}

pub fn add_pike_func(name: &str, type_str: &str, fun: unsafe extern "C" fn(i32) -> ())
{
  let func_name = CString::new(name).unwrap();
  let func_type = CString::new(type_str).unwrap();
  unsafe {
    pike_add_function2(func_name.as_ptr(),
            Some(fun),
            func_type.as_ptr(),
            0,
            OPT_SIDE_EFFECT|OPT_EXTERNAL_DEPEND);
  }
}
