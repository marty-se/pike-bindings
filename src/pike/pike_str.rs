use pike::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{pike_string, f_string_to_utf8, f_utf8_to_string, really_free_string,
    debug_make_shared_binary_string, svalue};
use serde::ser::*;

#[derive(Debug)]
pub struct PikeStringRef {
    ptr: *mut pike_string,
}

refcounted_type!(PikeStringRef, pike_string, DeferredStringDrop);

struct DeferredStringDrop {
    ptr: *mut pike_string
}

impl DropWithContext for DeferredStringDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                really_free_string(ptr);
            }
        }
    }
}

#[derive(Debug)]
pub struct PikeString<'ctx> {
    string_ref: PikeStringRef,
    ctx: &'ctx PikeContext
}

define_from_impls!(PikeStringRef, PikeString, PikeString, string_ref);

impl<'ctx> From<PikeString<'ctx>> for String {
    fn from(pikestr: PikeString) -> String {
        let ctx = pikestr.ctx;
        let pt = PikeThing::PikeString(pikestr.into());
        ctx.push_to_stack(pt);

        unsafe {
            f_string_to_utf8(1);
            let thing = ctx.pop_from_stack();
            match thing {
                PikeThing::PikeString(res) => {
                    let pikestr = res.ptr;
                    // pike_string.str is strangely enough signed so we need to
                    // transmute to get the type we need.
                    let slice: &[i8] =
                        ::std::slice::from_raw_parts(&((*pikestr).str[0]),
                        (*pikestr).len as usize);
                    let slice2: &[u8] = ::std::mem::transmute(slice);
                    let mut v: ::std::vec::Vec<u8> = ::std::vec::Vec::new();
                    v.extend_from_slice(slice2);
                    String::from_utf8(v).unwrap()
                }
                _ => {
                    panic!("string_to_utf8 returned wrong type");
                }
            }
        }
    }
}

impl<'a> From<&'a PikeString<'a>> for String {
    fn from(pikestr: &'a PikeString) -> String {
        let ctx = pikestr.ctx;
        let pt = PikeThing::PikeString(pikestr.clone().into());
        ctx.push_to_stack(pt);

        unsafe {
            f_string_to_utf8(1);
            let thing = ctx.pop_from_stack();
            match thing {
                PikeThing::PikeString(res) => {
                    let pikestr = res.ptr;
                    // pike_string.str is strangely enough signed so we need to
                    // transmute to get the type we need.
                    let slice: &[i8] =
                        ::std::slice::from_raw_parts(&((*pikestr).str[0]),
                        (*pikestr).len as usize);
                    let slice2: &[u8] = ::std::mem::transmute(slice);
                    let mut v: ::std::vec::Vec<u8> = ::std::vec::Vec::new();
                    v.extend_from_slice(slice2);
                    ::std::string::String::from_utf8(v).unwrap()
                }
                _ => {
                    panic!("string_to_utf8 returned wrong type");
                }
            }
        }
    }
}

impl<'ctx> FromWithCtx<'ctx, String> for PikeStringRef {
    fn from_with_ctx(s: String, ctx: &'ctx PikeContext) -> Self {
        let pike_str: PikeString = s.into_with_ctx(ctx);
        pike_str.into()
    }
}

impl<'ctx> FromWithCtx<'ctx, String> for PikeString<'ctx> {
    fn from_with_ctx(s: String, ctx: &'ctx PikeContext) -> Self {
        let raw_str_ref = unsafe { PikeStringRef::from_ptr(
            debug_make_shared_binary_string(
                s.as_ptr() as *const i8,
                s.len(),
            )) };
        let pt = PikeThing::PikeString(raw_str_ref);
        ctx.push_to_stack(pt);

        unsafe { f_utf8_to_string(1) };
        match ctx.pop_from_stack() {
            PikeThing::PikeString(str_ref) => {
                PikeString { string_ref: str_ref, ctx: ctx }
            }
            _ => {
                panic!("string_to_utf8 returned wrong type");
            }
        }
    }
}

impl<'ctx> PikeString<'ctx> {

    pub fn from_str_slice(s: &str, ctx: &'ctx PikeContext) -> Self {
        let raw_str_ref = unsafe { PikeStringRef::from_ptr(
            debug_make_shared_binary_string(
                s.as_ptr() as *const i8,
                s.len(),
            )) };

        let pt = PikeThing::PikeString(raw_str_ref);
        ctx.push_to_stack(pt);

        unsafe { f_utf8_to_string(1) };
        match ctx.pop_from_stack() {
            PikeThing::PikeString(str_ref) => {
                PikeString { string_ref: str_ref, ctx: ctx }
            }
            _ => {
                panic!("string_to_utf8 returned wrong type");
            }
        }
    }

    pub fn from_vec(v: Vec<u8>, ctx: &'ctx PikeContext) -> Self {
        let str_ref = unsafe { PikeStringRef::from_ptr(
            debug_make_shared_binary_string(
                v.as_ptr() as *const i8,
                v.len(),
            )) };
        PikeString { string_ref: str_ref, ctx: ctx }
    }

    pub fn from_vec_slice<'slice>(v: &'slice [u8], ctx: &'ctx PikeContext)
        -> Self {
        let str_ref = unsafe { PikeStringRef::from_ptr(
            debug_make_shared_binary_string(
                v.as_ptr() as *const i8,
                v.len(),
            )) };
        PikeString { string_ref: str_ref, ctx: ctx }
    }
}

impl<'a> Serialize for PikeString<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: String = self.into();
        serializer.serialize_str(&s)
    }
}
