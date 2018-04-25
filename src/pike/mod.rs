use ::bindings::*;
use std::os::raw::*;
use std::ptr;
use std::ffi::CString;

mod pike_function;
mod pike_svalue;
mod pike_str;

pub fn start_new_program(filename: &str, line: u32)
{
  unsafe {
    let fname = ::std::ffi::CString::new(filename).unwrap();
    debug_start_new_program(line as i64, fname.as_ptr());
  }
}

pub fn end_class(name: &str) {
  let class_name = ::std::ffi::CString::new(name).unwrap();
  unsafe {
    debug_end_class(class_name.as_ptr(), class_name.to_bytes().len() as isize, 0);
  }
}

pub fn add_pike_func(name: &str, type_str: &str, fun: unsafe extern "C" fn(i32) -> ())
{
  let func_name = CString::new(name).unwrap();
  let func_type = CString::new(type_str).unwrap();
  unsafe {
    pike_add_function2(func_name.as_ptr(),
            Some(fun),
            func_type.as_ptr(),
            0,
            OPT_SIDE_EFFECT|OPT_EXTERNAL_DEPEND);
  }
}

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
    let sval = svalue::get_from_stack (pos);
    return (&sval).into();
  }

  pub fn push_to_stack(&self) {
    let sval: svalue = self.into();
    sval.push_to_stack();
  }

  pub fn pop_from_stack() -> Self {
    let sval = svalue::pop_from_stack();
    return (&sval).into();
  }
}

pub struct PikeArray {
  array: *mut array
}

pub struct PikeFloat {
  float_number: f64
}

pub struct PikeFunction {
  pikeobj: PikeObject,
  fun_idx: c_ushort
}

pub struct PikeInt {
  integer: c_long
}

pub struct PikeMapping {
  mapping: *mut mapping
}

pub struct PikeMultiset {
  multiset: *mut multiset
}

pub struct PikeObject {
  object: *mut object
}

pub struct PikeString {
  pike_string: *mut pike_string
}

pub struct PikeProgram {
  program: *mut program
}

pub struct PikeType {
  pike_type: *mut pike_type
}

impl PikeFloat {
  pub fn new(f: f64) -> Self {
    PikeFloat { float_number: f }
  }
}

impl PikeInt {
  pub fn new(i: c_long) -> Self {
    PikeInt { integer: i }
  }
}

macro_rules! def_pike_type {
  ($rtype:ident, $ptype:ident, $free_func:ident) => (

  impl $rtype {
    pub fn new($ptype: *mut $ptype) -> Self {
      unsafe {
        (*$ptype).refs += 1;
      }
      $rtype { $ptype: $ptype }
    }
  }

  impl Clone for $rtype {
    fn clone(&self) -> Self {
      unsafe {
        let $ptype: *mut $ptype = self.$ptype;
        (*$ptype).refs += 1;
      }
      $rtype { $ptype: self.$ptype }
    }
  }

  impl Drop for $rtype {
    fn drop(&mut self) {
      unsafe {
        let $ptype: *mut $ptype = self.$ptype;
        (*$ptype).refs -= 1;
        if (*$ptype).refs == 0 {
          $free_func($ptype);
        }
      }
    }
  }
)}

def_pike_type!(PikeArray, array, really_free_array);
def_pike_type!(PikeMapping, mapping, really_free_mapping);
def_pike_type!(PikeMultiset, multiset, really_free_multiset);
def_pike_type!(PikeObject, object, schedule_really_free_object);
def_pike_type!(PikeString, pike_string, really_free_string);
def_pike_type!(PikeProgram, program, really_free_program);
def_pike_type!(PikeType, pike_type, really_free_pike_type);



