#![allow(non_camel_case_types)]
include!(concat!(env!("OUT_DIR"), "/sys-bindings.rs"));

use pike::*;
use ffi::*;

use ::std::os::raw::{c_char, c_int};
use ::std::ptr::{null_mut};
use ::std::fmt;
use ::std::error::Error;

fn describe_pike_error(pike_error: &PikeThing, ctx: &PikeContext) -> String {
    let pike_master = PikeObject::<()>::get_master(ctx);
    let desc_res = pike_master.call_func("describe_error", vec![pike_error]);
    if let Ok(pt) = desc_res {
        if let PikeThing::PikeString(s) = pt {
            return s.unwrap(ctx).into();
        }
    }
    "Failed to describe error".to_string()
}

#[derive(Debug)]
pub enum PikeError {
    Args(String),
    Generic(String),
    PikeError(String, PikeThing)
}

impl fmt::Display for PikeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PikeError::Args(reason) => {
                write!(f, "{}", reason)
            },
            PikeError::Generic(reason) => {
                write!(f, "{}", reason)
            },
            PikeError::PikeError(reason, _pike_err) => {
                write!(f, "{}", reason)
            }
        }
    }
}

impl<'a> Error for PikeError {
    fn description(&self) -> &str {
        "Pike error"
    }
}

impl From<String> for PikeError {
    fn from(s: String) -> Self {
        PikeError::Generic(s)
    }
}

/// Prepares an error message, i.e. pushes it as a Pike string on the
/// Pike stack in preparation for a pike_error() call. Note that the Rust
/// variable that owns the &str reference must be out of scope when pike_error()
/// is called (or std::mem::drop() called on it) if it's a local variable.
pub fn prepare_error_message(message: &str) {
    let mut msg_with_newline = String::from(message);
    let last_char = msg_with_newline.chars().last();
    if last_char != Some('\n') {
        msg_with_newline.push('\n');
    }
    let cstr = ::std::ffi::CString::new(msg_with_newline)
        .expect("Error message cannot contain NUL bytes");
    unsafe { ::ffi::push_text(cstr.as_ptr()) }
}

impl PikeContext {

    /// Throws a Pike error.
    /// Note: This function is unsafe and will longjump to the current Pike
    /// catch context. All Rust variables must be out of scope or dropped
    /// manually. The error message must have been set up earlier by
    /// prepare_error_message() (which pushes it on the Pike stack so it can
    /// be cleaned up by the Pike runtime.)
    pub unsafe fn pike_error(self) -> ! {
        let pt = self.get_from_stack(-1);
        ::std::mem::drop(self);
        match pt {
            PikeThing::PikeString(s) => {
                let ps: *mut pike_string = s.as_mut_ptr();
                let fmt_str: *const c_char = ::std::mem::transmute(&((*ps).str));
                ::ffi::Pike_error(fmt_str);
            }
            _ => {
                panic!("Unexpected type on stack");
            }
        }
        ::std::unreachable!();
    }

    /// Calls a closure and catches Pike errors that are thrown from it.
    /// NOTE: Make sure that your closure doesn't initialize Rust variables that
    /// may perform heap allocations, or that are in some other way dependent on
    /// being Dropped properly, before calling any Pike code that may throw
    /// Pike errors.
    pub fn catch_pike_error<F, TRes>(&self, closure: F) -> Result<TRes, PikeError>
    where F: FnOnce() -> TRes {
        let mut buf;
        unsafe {
            buf = JMP_BUF {
                previous: null_mut(),
                recovery: ::std::mem::zeroed(),
                frame_pointer: null_mut(),
                stack_pointer: 0,
                mark_sp: 0,
                severity: 0,
                onerror: null_mut()
            };
        }
        let setjmp_res: c_int;
        let call_res: Result<TRes, PikeError>;

        unsafe {
            // Set up Pike catch context in buf.
            init_recovery(&mut buf, 0);

            // FIXME: We should use _setjmp or sigsetjmp with a 0 second argument
            // on the respective relevant platforms to avoid restoring signal masks.
            // In practice it's probably unusual to change signal masks in Pike
            // code called by this mechanism, but...
            setjmp_res = setjmp(::std::mem::transmute(&mut (buf.recovery)));
        }

        if setjmp_res != 0 {
            // By definition of setjmp we will end up here if a Pike error is
            // thrown. The global variable throw_value (declared by Pike) will
            // contain the thrown svalue.
            let throw_val: svalue = unsafe { (&throw_value).clone(self) };
            let thrown: PikeThing = throw_val.into();

            unsafe {
                throw_value.sub_ref(self);
                throw_value.mark_free();
            };

            let desc = describe_pike_error(&thrown, self);
            call_res = Err(PikeError::PikeError(desc, thrown))
        } else {
            call_res = Ok(closure());
        }

        unsafe {
            (*Pike_interpreter_pointer).recoveries = buf.previous;
        }

        call_res
    }
}
