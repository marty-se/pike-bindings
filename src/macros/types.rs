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
                ::interpreter::drop_with_context(new_ref);
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
                ::interpreter::drop_with_context(new_ref);
            }
        }
    };
}
