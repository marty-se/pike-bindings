use ::serde::ser::*;

#[derive(Debug, Clone)]
pub struct PikeFloat {
    float_number: f64
}

impl PikeFloat {
    pub fn new(f: f64) -> Self {
        PikeFloat { float_number: f }
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
