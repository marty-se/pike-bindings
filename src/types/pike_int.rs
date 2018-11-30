use ::ffi::*;
use ::serde::ser::*;
use std::os::raw::c_long;

#[derive(Debug, Clone)]
pub struct PikeInt {
    pub integer: c_long
}

impl PikeInt {
    pub fn new(i: c_long) -> Self {
        PikeInt { integer: i }
    }
}

impl<'a> From<&'a PikeInt> for svalue {
    fn from (i: &PikeInt) -> Self {
        let a = ::ffi::anything { integer: i.integer };
        let t = ::ffi::svalue__bindgen_ty_1__bindgen_ty_1 {
            type_: PIKE_T_INT as ::std::os::raw::c_ushort, subtype: 0
        };
        let tu = ::ffi::svalue__bindgen_ty_1 {t: t};
        ::ffi::svalue {u: a, tu: tu}
    }
}

macro_rules! gen_from_type {
    ($inttype: ident) => {
        impl From<$inttype> for PikeInt {
            fn from(i: $inttype) -> PikeInt {
                return PikeInt::new(i.into());
            }
        }
        impl From<PikeInt> for $inttype {
            fn from (i: PikeInt) -> $inttype {
                return i.integer as $inttype;
            }
        }
        impl<'a> From<&'a PikeInt> for $inttype {
            fn from (i: &'a PikeInt) -> $inttype {
                return i.integer as $inttype;
            }
        }
    };
}

// FIXME: Convert overflowing types to Pike bignums?
//gen_from_type!(u64);
gen_from_type!(u32);
gen_from_type!(u16);
gen_from_type!(u8);

gen_from_type!(i64);
gen_from_type!(i32);
gen_from_type!(i16);
gen_from_type!(i8);

impl Serialize for PikeInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_i64(self.integer)
    }
}
