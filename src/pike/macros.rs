macro_rules! define_from_impls {
    ($reftype:ident, $type:ident, $pikethingtype:ident, $refname:ident) => (
        impl<'ctx> From<$type<'ctx>> for $reftype {
            fn from(t: $type) -> Self {
                t.$refname
            }
        }

        impl<'ctx> From<$type<'ctx>> for PikeThing {
            fn from(t: $type) -> Self {
                PikeThing::$pikethingtype(t.into())
            }
        }

        impl<'ctx, 'a> From<&'a $type<'ctx>> for svalue {
            fn from(t_ref: &$type) -> Self {
                let t: $type = t_ref.clone();
                t.into()
            }
        }

        impl<'ctx> From<$type<'ctx>> for svalue {
            fn from(t: $type) -> Self {
                PikeThing::from(t).into()
            }
        }

        impl<'ctx> Clone for $type<'ctx> {
            fn clone(&self) -> Self {
                Self { $refname: self.$refname.clone(self.ctx), ctx: self.ctx }
            }
        }

        impl From<$reftype> for PikeThing {
            fn from(f: $reftype) -> Self {
                PikeThing::$pikethingtype(f)
            }
        }
    )
}
/*
macro_rules! define_into_svalue {
    ($reftype:ident, $piketype:ident) => (
        impl From<$reftype> for svalue {
            fn from(s: $svalue) -> Self {
                // The reference is simply transferred, so we'll
                let res = Self { $piketype: s.u.$piketype };
                ::std::mem::forget(s);
                res
            }
        }
    )
}
*/
/*
macro_rules! def_pike_type {
  ($reftype:ident, $ptype:ident, $anything_type:ident, $svalue_type:ident,
   $free_func:ident) => (

    impl $reftype {
        pub fn new($ptype: *mut $ptype, _ctx: &PikeContext) -> Self {
            unsafe {
                (*$ptype).refs += 1;
            }
            $reftype { $ptype: $ptype }
        }

        // Cannot implement regular Clone trait since we need a &PikeContext
        // argument.
        pub fn clone(&self, ctx: &PikeContext) -> Self {
            Self::new(self.$ptype, ctx)
        }
    }
*/
/*
    impl Clone for $reftype {
        fn clone(&self) -> Self {
            unsafe {
                let $ptype: *mut $ptype = self.$ptype;
                (*$ptype).refs += 1;
            }
            $reftype { $ptype: self.$ptype }
        }
    }
*/
/*
    impl Drop for $reftype {
        fn drop(&mut self) {
          // FIXME: Push object to a release pool that is processed somewhere,
          // perhaps in the Drop trait impl for PikeContext?
/*
            unsafe {
                let $ptype: *mut $ptype = self.$ptype;
                (*$ptype).refs -= 1;
                if (*$ptype).refs == 0 {
                    $free_func($ptype);
                }
            }
            */
        }
    }
)}
*/