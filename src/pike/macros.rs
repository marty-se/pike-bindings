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

        impl<'ctx, 'a> FromWithCtx<'ctx, &'a $reftype> for $type<'ctx> {
            fn from_with_ctx($refname: &$reftype, ctx: &'ctx PikeContext) -> Self {
                Self { $refname: $refname.clone_with_ctx(ctx), ctx }
            }
        }
    )
}

macro_rules! define_from_impls_with_storage {
    ($reftype:ident, $type:ident, $pikethingtype:ident, $refname:ident) => (
        impl<'ctx, TStorage> From<$type<'ctx, TStorage>> for $reftype<TStorage> {
            fn from(t: $type<TStorage>) -> Self {
                t.$refname
            }
        }

        impl<'ctx, TStorage> From<$type<'ctx, TStorage>> for PikeThing {
            fn from(t: $type<TStorage>) -> Self {
                $reftype::from(t).into()
            }
        }

        impl<'ctx, 'a, TStorage> From<&'a $type<'ctx, TStorage>> for svalue {
            fn from(t_ref: &$type<TStorage>) -> Self {
                let t: $type<TStorage> = t_ref.clone();
                t.into()
            }
        }

        impl<'ctx, TStorage> From<$type<'ctx, TStorage>> for svalue {
            fn from(t: $type<TStorage>) -> Self {
                PikeThing::from(t).into()
            }
        }

        impl<'ctx, TStorage> Clone for $type<'ctx, TStorage> {
            fn clone(&self) -> Self {
                Self {
                    $refname: self.$refname.clone_with_ctx(self.ctx),
                    ctx: self.ctx
                }
            }
        }

        impl<TStorage> From<$reftype<TStorage>> for PikeThing {
            fn from(f: $reftype<TStorage>) -> Self {
                let untyped_obj: $reftype<()> = unsafe {
                    std::mem::transmute(f)
                };
                PikeThing::$pikethingtype(untyped_obj)
            }
        }

        impl<'ctx, TStorage> FromWithCtx<'ctx, $reftype<TStorage>>
            for $type<'ctx, TStorage> {
            fn from_with_ctx($refname: $reftype<TStorage>,
                ctx: &'ctx PikeContext) -> Self {
                Self { $refname, ctx }
            }
        }

        impl<'ctx, 'a, TStorage> FromWithCtx<'ctx, &'a $reftype<TStorage>>
            for $type<'ctx, TStorage> {
            fn from_with_ctx($refname: &$reftype<TStorage>,
                ctx: &'ctx PikeContext) -> Self {
                Self { $refname: $refname.clone_with_ctx(ctx), ctx }
            }
        }
    )
}
