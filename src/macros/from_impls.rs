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
