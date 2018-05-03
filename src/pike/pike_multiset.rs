use ::bindings::*;

pub struct PikeMultiset {
  multiset: *mut multiset
}

def_pike_type!(PikeMultiset, multiset, multiset, PIKE_T_MULTISET, really_free_multiset);