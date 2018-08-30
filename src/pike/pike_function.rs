use ::pike::*;
use ::pike::error::catch_pike_error;
use ::ffi::PIKE_T_FUNCTION;
use std::os::raw::c_ushort;

#[derive(Clone, Debug)]
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

impl From<PikeFunction> for Box<Fn(PikeThing) -> Result<PikeThing, PikeError>> {
  fn from (f: PikeFunction) -> Self {
    let res: Box<Fn(PikeThing) -> Result<PikeThing, PikeError>> = Box::new(move |arg1| f.call(vec![&arg1]));
    res
  }
}

impl PikeFunction {
  pub fn new(object: *mut object, fun_idx: c_ushort) -> Self {
    let pikeobj = PikeObject::<()>::new(object);
    PikeFunction { pikeobj: pikeobj, fun_idx: fun_idx }
  }

  pub fn call(&self, args: Vec<&PikeThing>) -> Result<PikeThing, PikeError> {
    for a in &args {
      a.push_to_stack();
    }
    let mut func: svalue = self.into();
    let num_args = args.len() as i32;
    catch_pike_error(|| {
        unsafe {
            apply_svalue(&mut func, num_args);
        }
        PikeThing::pop_from_stack()
    })
  }
}
