use ::pike::*;
use ::serde::ser::*;
use ::serde::*;

use std::fmt;

use serde::de::{Visitor, MapAccess, SeqAccess};

/// The `PikeThing` type. Equivalent to Pike's `svalue` type, with Rust idioms.
#[derive(Debug)]
pub enum PikeThing {
  Array(PikeArray),
  Float(PikeFloat),
  Function(PikeFunction),
  Int(PikeInt),
  Mapping(PikeMapping),
  Multiset(PikeMultiset),
  Object(PikeObject),
  PikeString(PikeString),
  Program(PikeProgram),
  Type(PikeType)
}

impl PikeThing {
  pub fn get_from_stack (pos: isize) -> Self
  {
    let sval: &svalue;
    unsafe {
      sval = &*(*Pike_interpreter_pointer).stack_pointer.offset(pos);
    }
    return sval.into();
  }

  pub fn push_to_stack(&self) {
    let sval: svalue = self.into();
    sval.add_ref();
    unsafe {
      let sp = (*Pike_interpreter_pointer).stack_pointer;
      ptr::write(sp, sval);
      (*Pike_interpreter_pointer).stack_pointer = sp.offset(1);
    }
  }

  pub fn pop_from_stack() -> Self {
    // Ref is transferred, so we won't subtract refs.
    let sval: &svalue;
    let res: PikeThing;
    unsafe {
      (*Pike_interpreter_pointer).stack_pointer = (*Pike_interpreter_pointer).stack_pointer.offset(-1);
      let sp = (*Pike_interpreter_pointer).stack_pointer;
      sval = &*sp;
      res = sval.into();
      ptr::write(sp, svalue::undefined());
    }
    return res;
  }

  pub fn pop_n_elems(num_elems: usize) {
    unsafe {
      let mut sp = (*Pike_interpreter_pointer).stack_pointer;
      for _ in 0..num_elems {
        sp = sp.offset(-1);
        ptr::write(sp, svalue::undefined());
      }
    }
  }

  pub fn undefined() -> Self {
    let sval = svalue::undefined();
    let res: PikeThing = (&sval).into();
    return res;
  }
}

impl Serialize for PikeThing {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: ::serde::Serializer {
    match self {
      PikeThing::Array(a) => { a.serialize(serializer) }
      PikeThing::Mapping(m) => { m.serialize(serializer) }
      PikeThing::Multiset(m) => { m.serialize(serializer) }
      PikeThing::PikeString(s) => { s.serialize(serializer) }
      PikeThing::Int(i) => { i.serialize(serializer) }
      PikeThing::Float(f) => { f.serialize(serializer) }
      _ => Err(ser::Error::custom("Unsupported Pike type"))
    }
  }
}

struct PikeThingVisitor;

impl<'de> Visitor<'de> for PikeThingVisitor {
    type Value = PikeThing;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("something serializeable to a Pike thing")
    }

    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(value.into())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
        Ok(PikeThing::PikeString(v.into()))
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where M: MapAccess<'de>,
    {
        let m = PikeMapping::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            m.insert(&key, &value);
        }

        Ok(PikeThing::Mapping(m))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where A: SeqAccess<'de>,
    {
        let mut a = PikeArray::with_capacity(seq.size_hint().unwrap_or(0));

        while let Some(value) = seq.next_element()? {
            a.append(&value)
        }
        Ok(PikeThing::Array(a))
    }
}

impl<'de> Deserialize<'de> for PikeThing {
  fn deserialize<D>(deserializer: D) -> Result<PikeThing, D::Error>
  where D: Deserializer<'de> {
    deserializer.deserialize_any(PikeThingVisitor)
  }
}

impl From<()> for PikeThing {
  fn from(_: ()) -> PikeThing {
    return PikeThing::undefined();
  }
}

macro_rules! gen_from_type_int {
  ($inttype: ident) => {
    impl From<$inttype> for PikeThing {
      fn from(i: $inttype) -> PikeThing {
      return PikeThing::Int(i.into());
      }
    }
  };
}

gen_from_type_int!(u64);
gen_from_type_int!(u32);
gen_from_type_int!(u16);
gen_from_type_int!(u8);

gen_from_type_int!(i64);
gen_from_type_int!(i32);
gen_from_type_int!(i16);
gen_from_type_int!(i8);

macro_rules! gen_from_type_float {
  ($floattype: ident) => {
    impl From<$floattype> for PikeThing {
      fn from(f: $floattype) -> PikeThing {
      return PikeThing::Float(f.into());
      }
    }
  };
}

gen_from_type_float!(f64);
gen_from_type_float!(f32);


impl From<String> for PikeThing {
  fn from(s: String) -> Self {
    PikeThing::PikeString(s.into())
  }
}

impl<'a> From<&'a str> for PikeThing {
  fn from(s: &str) -> Self {
    PikeThing::PikeString(s.into())
  }
}

impl From<PikeString> for PikeThing {
    fn from(s: PikeString) -> Self {
        PikeThing::PikeString(s)
    }
}