use pike::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{pike_string, f_string_to_utf8, f_utf8_to_string, really_free_string,
    debug_make_shared_binary_string};
use serde::ser::*;

// Raw pointers (e.g. *mut pike_string) are not Send-safe by default.
// However, we know that Pike won't free the pike_string, leaving the pointer
// dangling, as long as we don't decrement the refcount we incremented in
// ::new().
unsafe impl Send for PikeStringRef {}
unsafe impl Send for DeferredStringDrop {}

#[derive(Debug)]
pub struct PikeStringRef {
    pike_string: *mut pike_string,
}

impl PikeStringRef {
    pub fn new(string: *mut pike_string, _ctx: &PikeContext) -> Self {
        unsafe {
            (*string).refs += 1;
        }
        Self { pike_string: string }
    }

    pub fn new_without_ref(string: *mut pike_string) -> Self {
        Self { pike_string: string }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.pike_string, ctx)
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) -> PikeString<'ctx> {
        PikeString { string_ref: self, ctx: ctx }
    }

    pub fn as_mut_ptr(&self) -> *mut pike_string {
        self.pike_string
    }
}

impl Drop for PikeStringRef {
    fn drop(&mut self) {
        // This may be called anywhere, but we may only decrease refs (and
        // potentially free) while the interpreter lock is held. However, we
        // don't want to acquire the lock here, both for performance reasons and
        // potential locking order predictability issues. Instead, we'll
        // transfer the pointer to a new struct that we send to
        // drop_with_context().
        // Note: our contribution to the reference counter of the raw
        // pike_string struct is transferred to the DeferredStringDrop here.
        let new_ref = DeferredStringDrop { pike_string: self.pike_string };
        ::pike::interpreter::drop_with_context(new_ref);
    }
}

struct DeferredStringDrop {
    pike_string: *mut pike_string
}

impl DropWithContext for DeferredStringDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.pike_string;
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

impl<'ctx> Clone for PikeString<'ctx> {
    fn clone(&self) -> Self {
        Self { string_ref: self.string_ref.clone(self.ctx), ctx: self.ctx }
    }
}

impl<'ctx> From<PikeString<'ctx>> for PikeStringRef {
    fn from(pikestr: PikeString) -> Self {
        pikestr.string_ref
    }
}

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
                    let pikestr = res.pike_string;
                    // pike_string.str is strangely enough signed so we need to
                    // transmute to get the type we need.
                    let slice: &[i8] =
                        ::std::slice::from_raw_parts(&((*pikestr).str[0]), (*pikestr).len as usize);
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
                    let pikestr = res.pike_string;
                    // pike_string.str is strangely enough signed so we need to
                    // transmute to get the type we need.
                    let slice: &[i8] =
                        ::std::slice::from_raw_parts(&((*pikestr).str[0]), (*pikestr).len as usize);
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

impl<'ctx> PikeString<'ctx> {
    pub fn from_string(s: String, ctx: &'ctx PikeContext) -> Self {
        let raw_str_ref = PikeStringRef::new(unsafe {
            debug_make_shared_binary_string(
                s.as_ptr() as *const i8,
                s.len(),
            )}, ctx);

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

    pub fn from_str_slice(s: &str, ctx: &'ctx PikeContext) -> Self {
        let raw_str_ref = PikeStringRef::new(unsafe {
            debug_make_shared_binary_string(
                s.as_ptr() as *const i8,
                s.len(),
            )}, ctx);

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
        let str_ref = PikeStringRef::new(unsafe {
            debug_make_shared_binary_string(
                v.as_ptr() as *const i8,
                v.len(),
            )}, ctx);
        PikeString { string_ref: str_ref, ctx: ctx }
    }

    pub fn from_vec_slice<'slice>(v: &'slice [u8], ctx: &'ctx PikeContext) ->
        Self {
        let str_ref = PikeStringRef::new(unsafe {
            debug_make_shared_binary_string(
                v.as_ptr() as *const i8,
                v.len(),
            )}, ctx);
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
