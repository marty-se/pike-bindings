use ::types::type_deps::*;
use ::ffi::*;
use std::ffi::CString;
use ::std::marker::PhantomData;

#[derive(Debug)]
pub struct PikeProgramRef<TStorage>
where TStorage: Sized {
    ptr: *mut program,
    _phantom: PhantomData<TStorage>
}

refcounted_type_with_storage!(PikeProgramRef, program, DeferredProgramDrop);

struct DeferredProgramDrop {
    ptr: *mut program
}

impl DropWithContext for DeferredProgramDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                really_free_program(ptr);
            }
        }
    }
}

#[derive(Debug)]
pub struct PikeProgram<'ctx, TStorage>
where TStorage: Sized {
    program_ref: PikeProgramRef<TStorage>,
    ctx: &'ctx PikeContext
}

define_from_impls_with_storage!(PikeProgramRef, PikeProgram, Program,
    program_ref);

impl<'ctx, 'a,  TStorage> From<&'a PikeProgram<'ctx, TStorage>>
for PikeProgramRef<TStorage> {
    fn from(prog: &PikeProgram<'ctx, TStorage>) -> Self {
        prog.program_ref.clone_with_ctx(prog.ctx)
    }
}

macro_rules! storage_type {
    () => {
        *mut TStorage
    };
}

impl<'ctx, TStorage> PikeProgram<'ctx, TStorage> {
    pub unsafe fn from_ref(program_ref: PikeProgramRef<TStorage>,
        ctx: &'ctx PikeContext) -> Self {
        Self { program_ref: program_ref, ctx: ctx }
    }

    /// Instantiates a new program by finishing the current compilation unit.
    pub fn finish_program(ctx: &'ctx PikeContext) -> Self {
        let new_prog_ptr: *mut program;
        unsafe {
            new_prog_ptr = debug_end_program();
            let prog_ref =
                PikeProgramRef::<TStorage>::from_ptr_add_ref(new_prog_ptr, ctx);
            prog_ref.into_with_ctx(ctx)
        }
    }

    #[allow(clippy::cast_ptr_alignment)]
    pub fn clone_object(&self, data: TStorage)
      -> Result<PikeObject<TStorage>, PikeError> {
        self.ctx.catch_pike_error(|| {
            let obj: *mut object;
            let res_obj: PikeObject<TStorage>;
            let storage_ptr: *mut storage_type!();
            unsafe {
                obj = debug_clone_object(self.program_ref.ptr, 0);
                storage_ptr = (*obj).storage as *mut storage_type!();
                res_obj = PikeObjectRef::<TStorage>::from_ptr(obj)
                    .into_with_ctx(self.ctx);
                *storage_ptr = Box::into_raw(Box::new(data));
            }

            res_obj
        })
    }

    /// Returns the program that is currently being compiled.
    pub fn current_compilation(ctx: &'ctx PikeContext) -> Self {
        unsafe {
            let prog_ptr = (*Pike_compiler).new_program;
            PikeProgramRef::<TStorage>::from_ptr_add_ref(prog_ptr, ctx)
                .into_with_ctx(ctx)
        }
    }

    /// Adds the provided program to the program currently being compiled,
    /// with the provided name.
    pub fn add_program_constant(_ctx: &PikeContext, name: &str, prog: &Self) {
        let cname = ::std::ffi::CString::new(name).unwrap();
        unsafe {
            add_program_constant(cname.as_ptr(), prog.program_ref.ptr, 0);
        }
    }

    /// Adds a function to the program currently being compiled.
    pub fn add_pike_func(_ctx: &PikeContext, name: &str, type_str: &str,
        fun: unsafe extern "C" fn(i32) -> ())
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

    pub fn start_new_program(_ctx: &PikeContext,
        filename: &str, line: u32) {
        let fname = ::std::ffi::CString::new(filename).unwrap();
        unsafe {
            debug_start_new_program(line.into(), fname.as_ptr());
            low_add_storage(::std::mem::size_of::<storage_type!()>(),
                ::std::mem::align_of::<storage_type!()>(), 0);
            pike_set_prog_event_callback(Some(Self::prog_event_callback));
        }
    }

    // low_add_storage ensures that object storage (i.e. frame_ptr.current_storage)
    // is aligned properly, so we'll ignore the clippy lint here.
    #[allow(clippy::cast_ptr_alignment)]
    unsafe extern "C" fn prog_event_callback(event: i32) {
        match event as u32 {
            PROG_EVENT_INIT => {
                let frame_ptr = *(*Pike_interpreter_pointer).frame_pointer;
                let storage_ptr =
                    frame_ptr.current_storage as *mut storage_type!();
                *storage_ptr = ::std::ptr::null_mut();
            },
            PROG_EVENT_EXIT => {
                let frame_ptr = *(*Pike_interpreter_pointer).frame_pointer;
                let storage_ptr =
                    frame_ptr.current_storage as *mut storage_type!();
                if !(*storage_ptr).is_null() {
                    // Transfer ownership of pointer to Box, and drop it.
                    Box::from_raw(*storage_ptr);
                    *storage_ptr = ::std::ptr::null_mut();
                }
            },
            _ => {}
        }
    }
}
