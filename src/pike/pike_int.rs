use ::bindings::*;
use std::os::raw::c_long;

pub struct PikeInt {
  integer: c_long
}

impl PikeInt {
  pub fn new(i: c_long) -> Self {
    PikeInt { integer: i }
  }
}

impl<'a> From<&'a PikeInt> for svalue {
  fn from (i: &PikeInt) -> Self {
    let a = ::bindings::anything { integer: i.integer };
    let t = ::bindings::svalue__bindgen_ty_1__bindgen_ty_1 {
      type_: PIKE_T_INT as ::std::os::raw::c_ushort, subtype: 0 };
    let tu = ::bindings::svalue__bindgen_ty_1 {t: t};
    return ::bindings::svalue {u: a, tu: tu};
  }
}
