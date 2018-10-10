use ::std::rc::Rc;
use ::std::marker::PhantomData;
use ::std::os::raw::c_void;
use ::lazy_static::*;
use ::std::sync::Mutex;
use ::std::cell::RefCell;

use ::pike::*;
use ::ffi::*;

pub trait DropWithContext {
    fn drop_with_context(&self, ctx: &PikeContext);
}

struct DroppableContainer {
    droppable_box: Box<DropWithContext + Send>
}

lazy_static! {
    static ref DEFERRED_RELEASES: Mutex<Vec<DroppableContainer>> =
        Mutex::new(Vec::new());
}

pub fn drop_with_context<D>(droppable: D)
where D: DropWithContext + Send + 'static {
    let mut guard = DEFERRED_RELEASES.lock().expect("Mutex lock failed");

    let vec = &mut *guard;
    vec.push(DroppableContainer { droppable_box: Box::new(droppable) });
}

struct CallbackContext<F, TRes>
where F: FnOnce(PikeContext) -> TRes {
    cb: RefCell<Option<F>>,
    res: Option<TRes>
}

extern "C" fn call_with_interpreter_cb<F, TRes>(cb_ptr: *mut c_void)
where F: FnOnce(PikeContext) -> TRes {
    let cb_ctx: &mut CallbackContext<F, TRes> =
        unsafe { ::std::mem::transmute(cb_ptr) };

    match cb_ctx.cb.replace(None) {
        Some(cb) => {
            let ctx = PikeContext { no_send: PhantomData };
            cb_ctx.res = Some(cb(ctx));
        }
        None => {}
    }
}

#[derive(Debug)]
pub struct PikeContext {
    // Hack to opt out of the Send trait on stable Rust
    no_send: PhantomData<Rc<()>>
}

impl PikeContext {
    pub fn release(self) -> CtxReleased {
        let thread_state;

        // Make sure that self.drop() is called before the interpreter lock is
        // released.
        ::std::mem::drop(self);
        unsafe {
            thread_state = (*::ffi::Pike_interpreter_pointer).thread_state;
            ::ffi::pike_threads_allow(thread_state);
        };
        CtxReleased { no_send: PhantomData, thread_state: thread_state }
    }

    pub fn call_with_context<F, TRes>(closure: F) -> TRes
        where F: FnOnce(Self) -> TRes {
            let mut cb_ctx = CallbackContext {
                cb: RefCell::new(Some(closure)),
                res: None
            };
            unsafe {
                ::ffi::call_with_interpreter(Some(call_with_interpreter_cb::<F, TRes>),
                    &mut cb_ctx as *mut _ as *mut c_void);
            };
            cb_ctx.res.unwrap()
            /*
        let callback = || {
            let ctx = Self {no_send: PhantomData};
            closure(ctx);
        };
        call_helper(callback);
        */
    }
}

impl Drop for PikeContext {
    fn drop(&mut self) {
        let mut guard = DEFERRED_RELEASES.lock().expect("Mutex lock failed");
        let vec: &mut Vec<DroppableContainer> = &mut *guard;
        vec.drain(0..).for_each(|container| {
            let droppable = &*container.droppable_box;
            droppable.drop_with_context(self);
        });
    }
}


pub struct CtxReleased {
    // hack to opt out of Send on stable rust, which doesn't
    // have negative impls
    no_send: PhantomData<Rc<()>>,
    thread_state: *mut ::ffi::thread_state
}

impl Drop for CtxReleased {
    fn drop(&mut self) {
        unsafe {
            ::ffi::pike_threads_disallow(self.thread_state);
        }
    }
}


impl PikeContext {
    /// Returns a PikeThing from the Pike stack without popping it.
    pub fn get_from_stack (&self, pos: isize) -> PikeThing
    {
        let sval: &svalue;
        unsafe {
            sval = &(*(*Pike_interpreter_pointer).stack_pointer.offset(pos));
        }
        return sval.clone(self).into();
    }

    /// Pushes a PikeThing to the Pike stack.
    pub fn push_to_stack(&self, thing: PikeThing) {
        let sval: svalue = thing.into();
        unsafe {
            let sp = (*Pike_interpreter_pointer).stack_pointer;
            ptr::write(sp, sval);
            (*Pike_interpreter_pointer).stack_pointer = sp.offset(1);
        }
    }

    /// Pops the top value from the Pike stack and returns it as a PikeThing.
    pub fn pop_from_stack(&self) -> PikeThing {
        // Ref is transferred, so we won't subtract refs.
        let res: PikeThing;
        unsafe {
            (*Pike_interpreter_pointer).stack_pointer =
                (*Pike_interpreter_pointer).stack_pointer.offset(-1);
            let sp = (*Pike_interpreter_pointer).stack_pointer;
            let sval = &*sp;
            res = sval.clone(self).into();
            ptr::write(sp, svalue::undefined());
        }
        return res;
    }

    /// Pops and discards the specified number of entries from the Pike stack.
    pub fn pop_n_elems(&self, num_elems: usize) {
        unsafe {
            let mut sp = (*Pike_interpreter_pointer).stack_pointer;
            for _ in 0..num_elems {
                sp = sp.offset(-1);
                ptr::write(sp, svalue::undefined());
            }
        }
    }
}
