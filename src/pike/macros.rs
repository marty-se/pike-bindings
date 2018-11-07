macro_rules! refcounted_type {
    ($reftype:ident, $rawtype:ident, $drophandler:ident) => {
        // Raw pointers (e.g. *mut array) are not Send-safe by default.
        // However, we know that Pike won't free the array, leaving the pointer
        // dangling, as long as we don't decrement the refcount we incremented in
        // ::new().
        unsafe impl Send for $reftype {}
        unsafe impl Send for $drophandler {}

        impl Refcounted<$rawtype> for $reftype {
            unsafe fn from_ptr<'ctx>(ptr: *mut $rawtype) -> Self {
                Self { ptr }
            }

            unsafe fn from_ptr_add_ref<'ctx>(ptr: *mut $rawtype, _ctx: &'ctx PikeContext) -> Self {
                (*ptr).refs += 1;
                Self::from_ptr(ptr)
            }

            fn as_mut_ptr(&self) -> *mut $rawtype {
                self.ptr
            }
        }

        impl CloneWithCtx for $reftype {
            fn clone_with_ctx<'ctx>(&self, _ctx: &'ctx PikeContext) -> Self {
                unsafe { (*self.ptr).refs += 1 };
                Self { ptr: self.ptr }
            }
        }

        impl Drop for $reftype {
            fn drop(&mut self) {
                // This may be called anywhere, but we may only decrease refs (and
                // potentially free) while the interpreter lock is held. However, we
                // don't want to acquire the lock here, both for performance reasons and
                // potential locking order predictability issues. Instead, we'll
                // transfer the pointer to a new struct that we send to
                // drop_with_context().
                // Note: our contribution to the reference counter of the raw
                // mapping struct is transferred to the DeferredMappingDrop here.
                let new_ref = $drophandler { ptr: self.ptr };
                ::pike::interpreter::drop_with_context(new_ref);
            }
        }
    };
}

macro_rules! refcounted_type_with_storage {
    ($reftype:ident, $rawtype:ident, $drophandler: ident) => {
        // Raw pointers (e.g. *mut array) are not Send-safe by default.
        // However, we know that Pike won't free the array, leaving the pointer
        // dangling, as long as we don't decrement the refcount we incremented in
        // ::new().
        unsafe impl<TStorage> Send for $reftype<TStorage> {}
        unsafe impl Send for $drophandler {}

        impl<TStorage> Refcounted<$rawtype> for $reftype<TStorage> {
            unsafe fn from_ptr<'ctx>(ptr: *mut $rawtype) -> Self {
                Self { ptr, _phantom: PhantomData }
            }

            unsafe fn from_ptr_add_ref<'ctx>(ptr: *mut $rawtype, _ctx: &'ctx PikeContext) -> Self {
                (*ptr).refs += 1;
                Self::from_ptr(ptr)
            }

            fn as_mut_ptr(&self) -> *mut $rawtype {
                self.ptr
            }
        }

        impl<TStorage> CloneWithCtx for $reftype<TStorage> {
            fn clone_with_ctx<'ctx>(&self, _ctx: &'ctx PikeContext) -> Self {
                unsafe { (*self.ptr).refs += 1 };
                Self { ptr: self.ptr, _phantom: PhantomData }
            }
        }

        impl<TStorage> Drop for $reftype<TStorage> {
            fn drop(&mut self) {
                // This may be called anywhere, but we may only decrease refs (and
                // potentially free) while the interpreter lock is held. However, we
                // don't want to acquire the lock here, both for performance reasons and
                // potential locking order predictability issues. Instead, we'll
                // transfer the pointer to a new struct that we send to
                // drop_with_context().
                // Note: our contribution to the reference counter of the raw
                // mapping struct is transferred to the DeferredMappingDrop here.
                let new_ref = $drophandler { ptr: self.ptr };
                ::pike::interpreter::drop_with_context(new_ref);
            }
        }
    };
}

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
                Self { $refname: self.$refname.clone_with_ctx(self.ctx), ctx: self.ctx }
            }
        }

        impl From<$reftype> for PikeThing {
            fn from(f: $reftype) -> Self {
                PikeThing::$pikethingtype(f)
            }
        }

        impl<'ctx> FromWithCtx<'ctx, $reftype> for $type<'ctx> {
            fn from_with_ctx($refname: $reftype, ctx: &'ctx PikeContext) -> Self {
                Self { $refname, ctx }
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