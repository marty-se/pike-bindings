use ::bindings::*;

pub struct PikeArray {
  array: *mut array
}

def_pike_type!(PikeArray, array, array, PIKE_T_ARRAY, really_free_array);