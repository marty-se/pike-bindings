use ::pike::*;

pub struct PikeObject {
  object: *mut object
}

def_pike_type!(PikeObject, object, object, PIKE_T_OBJECT, schedule_really_free_object);

impl PikeObject {
  pub fn call_func(&self, func_name: &str, num_args: usize) -> Result<PikeThing, String> {
    let func_cstr = ::std::ffi::CString::new(func_name).map_err(|e| e.to_string())?;
    unsafe {
      safe_apply(self.object, func_cstr.as_ptr(), num_args as i32);
    }
    Ok(PikeThing::pop_from_stack())
  }

  pub fn as_mut_ptr(&self) -> *mut object {
    self.object
  }

  pub fn wrapped_object<'s, T>(&'s self) -> &'s mut Box<T> {
    unsafe {
      let fp = (*Pike_interpreter_pointer).frame_pointer;
      let ptr = (*fp).current_storage as *mut Box<T>;
      &mut *ptr
    }
  }
}
