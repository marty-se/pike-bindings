use ::bindings::*;
use ::bindings::PIKE_T_FLOAT;
use ::serde::ser::*;

pub struct PikeFloat {
  float_number: f64
}

impl PikeFloat {
  pub fn new(f: f64) -> Self {
    PikeFloat { float_number: f }
  }
}

impl<'a> From<&'a PikeFloat> for svalue {
  fn from (f: &PikeFloat) -> Self {
    let a = ::bindings::anything { float_number: f.float_number };
    let t = ::bindings::svalue__bindgen_ty_1__bindgen_ty_1 {
      type_: PIKE_T_FLOAT as ::std::os::raw::c_ushort, subtype: 0 };
    let tu = ::bindings::svalue__bindgen_ty_1 {t: t};
    return ::bindings::svalue {u: a, tu: tu};
  }
}

impl Serialize for PikeFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_f64(self.float_number)
    }
}