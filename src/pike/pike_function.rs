use ::pike::*;
use ::bindings::PIKE_T_FUNCTION;
use std::os::raw::c_ushort;

#[derive(Clone)]
pub struct PikeFunction {
  pikeobj: PikeObject<()>,
  fun_idx: c_ushort
}

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

impl<'a> From<&'a PikeFunction> for svalue {
  fn from (f: &PikeFunction) -> Self {
    let mut s: svalue = (&f.pikeobj).into();
    unsafe {
      s.tu.t.type_ = PIKE_T_FUNCTION as c_ushort;
      s.tu.t.subtype = f.fun_idx;
    }
    s
  }
}

impl PikeFunction {
  pub fn new(object: *mut object, fun_idx: c_ushort) -> Self {
    let pikeobj = PikeObject::<()>::new(object);
    PikeFunction { pikeobj: pikeobj, fun_idx: fun_idx }
  }

  pub fn call(&self, args: Vec<PikeThing>) -> PikeThing {
    for a in &args {
      a.push_to_stack();
    }
    let func = &PikeThing::from(self);
    unsafe {
      safe_apply_svalue(&mut svalue::from(func), args.len() as i32, 1);
    }
    PikeThing::pop_from_stack()
  }
}
