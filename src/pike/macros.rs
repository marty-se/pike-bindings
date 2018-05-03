macro_rules! def_pike_type {
  ($rtype:ident, $ptype:ident, $anything_type:ident, $svalue_type:ident, $free_func:ident) => (

  impl $rtype {
    pub fn new($ptype: *mut $ptype) -> Self {
      unsafe {
        (*$ptype).refs += 1;
      }
      $rtype { $ptype: $ptype }
    }
  }

  impl<'a> From<&'a $rtype> for ::bindings::svalue {
    fn from(t: &$rtype) -> Self {
      let a = ::bindings::anything { $anything_type: t.$ptype };
      let t = ::bindings::svalue__bindgen_ty_1__bindgen_ty_1 {
        type_: $svalue_type as ::std::os::raw::c_ushort, subtype: 0 };
      let tu = ::bindings::svalue__bindgen_ty_1 {t: t};
      return ::bindings::svalue {u: a, tu: tu};
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