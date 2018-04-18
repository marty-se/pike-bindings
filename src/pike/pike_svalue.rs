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
    let res: (anything, u32, u16) =
    match *pike_thing {
      PikeThing::Array(ref a) => {
        (anything { array: a.array }, PIKE_T_ARRAY, 0)
      }
      PikeThing::Float(ref f) => {
        (anything { float_number: f.float_number }, PIKE_T_FLOAT, 0)
      }
      PikeThing::Function(ref f) => {
        (anything { object: f.pikeobj.object }, PIKE_T_FUNCTION, f.fun_idx)
      }
      PikeThing::Int(ref i) => {
        (anything { integer: i.integer }, PIKE_T_INT, 0)
      }
      PikeThing::Mapping(ref m) => {
        (anything { mapping: m.mapping }, PIKE_T_MAPPING, 0)
      }
      PikeThing::Multiset(ref m) => {
        (anything { multiset: m.multiset }, PIKE_T_MULTISET, 0)
      }
      PikeThing::Object(ref o) => {
        (anything { object: o.object }, PIKE_T_OBJECT, 0)
      }
      PikeThing::PikeString(ref s) => {
        (anything { string: s.pike_string }, PIKE_T_STRING, 0)
      }
      PikeThing::Program(ref p) => {
        (anything { program: p.program }, PIKE_T_PROGRAM, 0)
      }
      PikeThing::Type(ref t) => {
        (anything { type_: t.pike_type }, PIKE_T_TYPE, 0)
      }
    };
    let t = svalue__bindgen_ty_1__bindgen_ty_1 { type_: res.1 as c_ushort, subtype: res.2 as c_ushort };
    let tu = svalue__bindgen_ty_1 {t: t};
    return svalue {u: res.0, tu: tu};
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

  pub fn push_to_stack (self)
  {
    self.add_ref();
    unsafe {
      let sp = (*Pike_interpreter_pointer).stack_pointer;
      ptr::write(sp, self);
      (*Pike_interpreter_pointer).stack_pointer = (*Pike_interpreter_pointer).stack_pointer.offset(1);
    }
  }

  pub fn pop_from_stack() -> Self {
    unsafe {
      // Ownership is transferred, so we won't subtract refs.
      (*Pike_interpreter_pointer).stack_pointer = (*Pike_interpreter_pointer).stack_pointer.offset(-1);
      let sval = &*(*Pike_interpreter_pointer).stack_pointer;
      return svalue { tu: sval.tu, u: sval.u };
    }
  }

  pub fn get_from_stack (stack_pos: isize) -> Self
  {
    let sval;
    unsafe {
      let sp = (*Pike_interpreter_pointer).stack_pointer.offset(stack_pos);
      sval = ptr::read(sp);
      sval.add_ref();
    }
    return sval;
  }
}

impl Clone for svalue {
  fn clone(&self) -> Self {
    self.add_ref();
    return svalue { tu: self.tu, u: self.u };
  }
}

impl Drop for svalue {
  fn drop(&mut self) {
    match self.sub_ref() {
      Some(r) => if r == 0 {
        unsafe {
          really_free_svalue(self);
        }
      }
      None => ()
    }
  }
}