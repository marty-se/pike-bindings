use ::ffi::*;
use ::pike::{PikeObject, PikeThing, PikeError, PikeContext};
use std::ffi::CString;
use ::std::marker::PhantomData;

pub use ::ffi::{low_add_storage, pike_set_prog_event_callback, PROG_EVENT_INIT, PROG_EVENT_EXIT};

#[derive(Debug)]
pub struct PikeProgramRef<TStorage>
where TStorage: Sized {
    program: *mut program,
    _phantom: PhantomData<TStorage>
}

impl<TStorage> PikeProgramRef<TStorage> {
    pub fn new(program: *mut program, _ctx: &PikeContext) -> Self {
        unsafe {
            (*program).refs += 1;
        }
        Self { program: program, _phantom: PhantomData }
    }

    pub unsafe fn new_without_ref(program: *mut program) -> Self {
        Self { program: program, _phantom: PhantomData }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.program, ctx)
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) ->
    PikeProgram<'ctx, TStorage> {
        PikeProgram { program_ref: self, ctx: ctx }
    }

    pub fn as_mut_ptr(&self) -> *mut program {
        self.program
    }
}

#[derive(Debug)]
pub struct PikeProgram<'ctx, TStorage>
where TStorage: Sized {
    program_ref: PikeProgramRef<TStorage>,
    ctx: &'ctx PikeContext
}

impl<'ctx, TStorage> Clone for PikeProgram<'ctx, TStorage> {
    fn clone(&self) -> Self {
        Self {
            program_ref: self.program_ref.clone(self.ctx),
            ctx: self.ctx
        }
    }
}

impl<'ctx, 'a,  TStorage> From<&'a PikeProgram<'ctx, TStorage>>
for PikeProgramRef<TStorage> {
    fn from(prog: &PikeProgram<'ctx, TStorage>) -> Self {
        prog.program_ref.clone(prog.ctx)
    }
}

impl<'ctx, TStorage> PikeProgram<'ctx, TStorage> {
    pub fn from_ptr(program: *mut program, ctx: &'ctx PikeContext) -> Self {
        let obj_ref = PikeProgramRef::new(program, ctx);
        Self::from_ref(obj_ref, ctx)
    }

    pub fn from_ref(program_ref: PikeProgramRef<TStorage>,
        ctx: &'ctx PikeContext) -> Self {
        Self { program_ref: program_ref, ctx: ctx }
    }

    /// Instantiates a new program by finishing the current compilation unit.
    pub fn finish_program(ctx: &'ctx PikeContext) -> Self {
        let new_prog_ptr: *mut program;
        unsafe {
            new_prog_ptr = debug_end_program();
        };
        let prog_ref = PikeProgramRef::new(new_prog_ptr, ctx);
        Self::from_ref(prog_ref, ctx)
    }

    pub fn clone_object(&self) -> Result<PikeObject<()>, PikeError> {
        self.ctx.catch_pike_error(|| {
              let obj: *mut object;
              unsafe {
                  obj = debug_clone_object(self.program_ref.program, 0);
              }
              PikeObject::<()>::from_ptr(obj, self.ctx)
        })
    }

    pub fn clone_object_with_data(&self, data: TStorage)
      -> Result<PikeObject<TStorage>, PikeError> {
          self.ctx.catch_pike_error(|| {
              let obj: *mut object;
              unsafe {
                  obj = debug_clone_object(self.program_ref.program, 0);
              }
              let res_obj = PikeObject::<TStorage>::from_ptr(obj, self.ctx);

              {
                  let storage = res_obj.wrapped();
                  unsafe {
                      ::std::ptr::write(storage, data);
                  }
              }
              res_obj
          })
    }

    /// Returns the program that is currently being compiled.
    pub fn current_compilation(ctx: &'ctx PikeContext) -> Self {
        let prog_ptr = unsafe { (*Pike_compiler).new_program };
        Self::from_ptr(prog_ptr, ctx)
    }

    /// Adds the provided program to the program currently being compiled,
    /// with the provided name.
    pub fn add_program_constant(name: &str, prog: Self) {
        let cname = ::std::ffi::CString::new(name).unwrap();
        unsafe {
            add_program_constant(cname.as_ptr(), prog.program_ref.program, 0);
        }
    }

    /// Adds a function to the program currently being compiled.
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

    // Calling this function is unsafe because object storage is zeroed on
    // initialization. Thus, clone_object_with_data must be used to initialize
    // storage when an object is instantiated.
    pub unsafe fn start_new_program(filename: &str, line: u32) {
        let fname = ::std::ffi::CString::new(filename).unwrap();
        debug_start_new_program(line as i64, fname.as_ptr());
        low_add_storage(::std::mem::size_of::<TStorage>(),
          ::std::mem::align_of::<TStorage>(), 0);
        pike_set_prog_event_callback(Some(Self::prog_event_callback));
    }

    pub fn start_new_program_with_default(filename: &str, line: u32)
    where TStorage: Default {
        unsafe {
            let fname = ::std::ffi::CString::new(filename).unwrap();
            debug_start_new_program(line as i64, fname.as_ptr());
            low_add_storage(::std::mem::size_of::<TStorage>(), ::std::mem::align_of::<TStorage>(), 0);
            pike_set_prog_event_callback(Some(Self::prog_event_callback_default));
        }
    }

    unsafe extern "C" fn prog_event_callback(event: i32) {
        match event as u32 {
            PROG_EVENT_INIT => {
                let storage_data: TStorage = ::std::mem::zeroed();
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

    unsafe extern "C" fn prog_event_callback_default(event: i32)
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
}

/*
impl<'a, TStorage> From<&'a PikeProgram<TStorage>> for ::ffi::svalue {
    fn from(t: &PikeProgram<TStorage>) -> Self {
        let a = ::ffi::anything { program: t.program };
        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_OBJECT as ::std::os::raw::c_ushort, subtype: 0 };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        return ::ffi::svalue {u: a, tu: tu};
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
*/

pub fn end_class(name: &str) {
  let class_name = ::std::ffi::CString::new(name).unwrap();
  unsafe {
    let prog: *mut program = debug_end_program();
    add_program_constant(class_name.as_ptr(), prog, 0);
  }
}

pub enum FnCallResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> Into<Result<T, E>> for FnCallResult<T, E> {

    fn into(self) -> Result<T, E> {
        match self {
            FnCallResult::Ok(v) => {
                Result::Ok(v)
            },
            FnCallResult::Err(e) => {
                Result::Err(e)
            }
        }
    }
}


impl<T, E> From<Result<T, E>> for FnCallResult<T, E> {

    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(v) => {
                FnCallResult::Ok(v)
            },
            Err(e) => {
                FnCallResult::Err(e)
            }
        }
    }
}

/*
impl<T, E> From<Result<T, E>> for FnCallResult<PikeThing, PikeError>
where
    T: Into<PikeThing>,
    E: Into<PikeError> {

    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(v) => {
                FnCallResult::Ok(v.into())
            },
            Err(e) => {
                FnCallResult::Err(e.into())
            }
        }
    }
}
*/

impl<T> From<T> for FnCallResult<PikeThing, PikeError>
where
    PikeThing: From<T> {

    fn from(val: T) -> Self {
        FnCallResult::Ok(PikeThing::from(val))
    }
}