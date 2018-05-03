use ::bindings::*;

pub struct PikeMapping {
  mapping: *mut mapping
}

def_pike_type!(PikeMapping, mapping, mapping, PIKE_T_MAPPING, really_free_mapping);
