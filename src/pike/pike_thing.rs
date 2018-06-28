use ::pike::*;
use ::serde::ser::*;
use ::serde::*;

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
