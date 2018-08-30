use ::ffi::*;
use ::ffi::PIKE_T_FLOAT;
use ::serde::ser::*;

#[derive(Debug)]
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
    let a = ::ffi::anything { float_number: f.float_number };
    let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
      type_: PIKE_T_FLOAT as ::std::os::raw::c_ushort, subtype: 0 };
    let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
    return ::ffi::svalue {u: a, tu: tu};
  }
}

macro_rules! gen_from_type {
  ($floattype: ident) => {
    impl From<$floattype> for PikeFloat {
      fn from(f: $floattype) -> PikeFloat {
        return PikeFloat::new(f as f64);
      }
    }
    impl From<PikeFloat> for $floattype {
      fn from(f: PikeFloat) -> $floattype {
        return f.float_number as $floattype;
      }
    }
    impl<'a> From<&'a PikeFloat> for $floattype {
      fn from(f: &'a PikeFloat) -> $floattype {
        return f.float_number as $floattype;
      }
    }
  };
}

gen_from_type!(f64);
gen_from_type!(f32);

impl Serialize for PikeFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_f64(self.float_number)
    }
}
