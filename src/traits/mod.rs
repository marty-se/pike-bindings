use ::interpreter::PikeContext;

pub(crate) trait Refcounted<TPtr>: Drop + CloneWithCtx {
    unsafe fn from_ptr(ptr: *mut TPtr) -> Self;
    unsafe fn from_ptr_add_ref(ptr: *mut TPtr, ctx: &PikeContext) -> Self;
    fn as_mut_ptr(&self) -> *mut TPtr;
}

pub trait CloneWithCtx: Sized {
    fn clone_with_ctx<'ctx>(&self, ctx: &'ctx PikeContext) -> Self;
}

pub trait FromWithCtx<'ctx, T>: Sized {
    fn from_with_ctx(_: T, ctx: &'ctx PikeContext) -> Self;
}

pub trait IntoWithCtx<'ctx, T>: Sized {
    fn into_with_ctx(self, ctx: &'ctx PikeContext) -> T;
}

pub(crate) trait DropWithContext {
    fn drop_with_context(&self, ctx: &PikeContext);
}
