use ::bindings::*;
use std::ffi::CString;

pub use ::bindings::{low_add_storage, pike_set_prog_event_callback, PROG_EVENT_INIT, PROG_EVENT_EXIT};

pub struct PikeProgram {
  program: *mut program
}

def_pike_type!(PikeProgram, program, program, PIKE_T_PROGRAM, really_free_program);

unsafe extern "C" fn prog_event_callback<T>(event: i32) {
    if event as u32 == PROG_EVENT_INIT {
        low_add_storage(::std::mem::size_of::<T>(), ::std::mem::align_of::<T>(), 0);
    }
}

pub fn start_new_program(filename: &str, line: u32)
{
  unsafe {
    let fname = ::std::ffi::CString::new(filename).unwrap();
    debug_start_new_program(line as i64, fname.as_ptr());
  }
}

pub fn end_class(name: &str) {
  let class_name = ::std::ffi::CString::new(name).unwrap();
  unsafe {
    debug_end_class(class_name.as_ptr(), class_name.to_bytes().len() as isize, 0);
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
