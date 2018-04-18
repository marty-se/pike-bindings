use ::pike::*;

impl From<PikeFunction> for PikeThing {
  fn from(f: PikeFunction) -> PikeThing {
    PikeThing::Function(f)
  }
}

impl<'a> From<&'a PikeFunction> for PikeThing {
  fn from(f: &PikeFunction) -> PikeThing {
    PikeThing::Function(f.clone())
  }
}

impl PikeFunction {
  pub fn new(object: *mut object, fun_idx: c_ushort) -> Self {
    let pikeobj = PikeObject::new(object);
    PikeFunction { pikeobj: pikeobj, fun_idx: fun_idx }
  }

  pub fn call(&self, args: Vec<PikeThing>) -> PikeThing {
    for a in &args {
      a.push_to_stack();
    }
    let func = PikeThing::from(self);
    unsafe {
      safe_apply_svalue(&mut svalue::from(&func), args.len() as i32, 1);
    }
    let res_sval = svalue::pop_from_stack();
    PikeThing::from(&res_sval)
  }
}

impl Clone for PikeFunction {
  fn clone(&self) -> Self {
    PikeFunction { pikeobj: self.pikeobj.clone(), fun_idx: self.fun_idx }
  }
}
