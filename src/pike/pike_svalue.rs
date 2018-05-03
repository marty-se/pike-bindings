use ::pike::*;

impl<'a> From<&'a svalue> for PikeThing {
  fn from (sval: &svalue) -> Self {
    let type_;
    let subtype;
    unsafe {
      type_ = sval.tu.t.type_;
      subtype = sval.tu.t.subtype;
      match type_ as u32 {
        PIKE_T_ARRAY => PikeThing::Array(PikeArray::new(sval.u.array)),
        PIKE_T_FLOAT => PikeThing::Float(PikeFloat::new(sval.u.float_number)),
        PIKE_T_FUNCTION => PikeThing::Function(PikeFunction::new(sval.u.object, subtype)),
        PIKE_T_INT => PikeThing::Int(PikeInt::new(sval.u.integer)),
        PIKE_T_MAPPING => PikeThing::Mapping(PikeMapping::new(sval.u.mapping)),
        PIKE_T_MULTISET => PikeThing::Multiset(PikeMultiset::new(sval.u.multiset)),
        PIKE_T_OBJECT => PikeThing::Object(PikeObject::new(sval.u.object)),
        PIKE_T_STRING => PikeThing::PikeString(PikeString::new(sval.u.string)),
        PIKE_T_PROGRAM => PikeThing::Program(PikeProgram::new(sval.u.program)),
        PIKE_T_TYPE => PikeThing::Type(PikeType::new(sval.u.type_)),
        _ => panic!("Unknown Pike type.")
      }
    }
  }
}

impl<'a> From<&'a PikeThing> for svalue {
  fn from (pike_thing: &PikeThing) -> Self {
    match *pike_thing {
      PikeThing::Array(ref a) => {
        a.into()
      }
      PikeThing::Float(ref f) => {
        f.into()
      }
      PikeThing::Function(ref f) => {
        f.into()
      }
      PikeThing::Int(ref i) => {
        i.into()
      }
      PikeThing::Mapping(ref m) => {
        m.into()
      }
      PikeThing::Multiset(ref m) => {
        m.into()
      }
      PikeThing::Object(ref o) => {
        o.into()
      }
      PikeThing::PikeString(ref s) => {
        s.into()
      }
      PikeThing::Program(ref p) => {
        p.into()
      }
      PikeThing::Type(ref t) => {
        t.into()
      }
    }
  }
}

impl svalue {
  pub fn add_ref(&self) -> Option<usize> {
    if self.refcounted_type() {
      unsafe {
        let r = self.u.dummy;
        (*r).refs += 1;
        return Some((*r).refs as usize);
      }
    }
    return None;
  }

  pub fn sub_ref(&self) -> Option<usize> {
    if self.refcounted_type() {
      unsafe {
        let r = self.u.dummy;
        (*r).refs -= 1;
        return Some((*r).refs as usize);
      }
    }
    return None;
  }

  fn type_(&self) -> u16 {
    unsafe {
      return self.tu.t.type_;
    }
  }

  #[allow(dead_code)]
  fn subtype(&self) -> u16 {
    unsafe {
      return self.tu.t.type_;
    }
  }

  fn refcounted_type(&self) -> bool {
    // Equivalent of REFCOUNTED_TYPE macro in svalue.h
    return (self.type_() & !(PIKE_T_ARRAY as u16 - 1)) != 0;
  }
}
