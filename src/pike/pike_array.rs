use ::pike::*;
use ::ffi::*;
use ::serde::ser::*;
use ::pike::interpreter::DropWithContext;

// Raw pointers (e.g. *mut array) are not Send-safe by default.
// However, we know that Pike won't free the array, leaving the pointer
// dangling, as long as we don't decrement the refcount we incremented in
// ::new().
unsafe impl Send for PikeArrayRef {}
unsafe impl Send for DeferredArrayDrop {}

#[derive(Debug)]
pub struct PikeArrayRef {
    array: *mut array
}

impl PikeArrayRef {
    pub fn new(array: *mut array, _ctx: &PikeContext) -> Self {
        unsafe {
            (*array).refs += 1;
        }
        Self { array: array }
    }

    pub fn new_without_ref(array: *mut array) -> Self {
        Self { array: array }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.array, ctx)
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) -> PikeArray<'ctx> {
        PikeArray { array_ref: self, ctx: ctx }
    }

    pub fn as_mut_ptr(&self) -> *mut array {
        self.array
    }
}

impl Drop for PikeArrayRef {
    fn drop(&mut self) {
        // This may be called anywhere, but we may only decrease refs (and
        // potentially free) while the interpreter lock is held. However, we
        // don't want to acquire the lock here, both for performance reasons and
        // potential locking order predictability issues. Instead, we'll
        // transfer the pointer to a new struct that we send to
        // drop_with_context().
        // Note: our contribution to the reference counter of the raw
        // mapping struct is transferred to the DeferredMappingDrop here.
        let new_ref = DeferredArrayDrop { array: self.array };
        ::pike::interpreter::drop_with_context(new_ref);
    }
}

struct DeferredArrayDrop {
    array: *mut array
}

impl DropWithContext for DeferredArrayDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.array;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                really_free_array(ptr);
            }
        }
    }
}

#[derive(Debug)]
pub struct PikeArray<'ctx> {
    array_ref: PikeArrayRef,
    ctx: &'ctx PikeContext
}

define_from_impls!(PikeArrayRef, PikeArray, Array, array_ref);

impl<'ctx> PikeArray<'ctx> {
    pub fn from_ref(array_ref: PikeArrayRef, ctx: &'ctx PikeContext) -> Self {
        Self { array_ref: array_ref, ctx: ctx }
    }

    /// Returns an empty array with a pre-allocated capacity (but 0 size).
    pub fn with_capacity(capacity: usize, ctx: &'ctx PikeContext) -> Self {
        // FIXME: Is a ref added implicitly?
        let new_array = unsafe { real_allocate_array(0, capacity as isize) };
        PikeArray {
            array_ref: PikeArrayRef {
                array: new_array
            },
            ctx: ctx
        }
    }

    /// Returns an array with the specified size.
    pub fn with_size(size: usize, ctx: &'ctx PikeContext) -> Self {
        let new_array = unsafe { real_allocate_array(size as isize, 0) };
        PikeArray {
            array_ref: PikeArrayRef {
                array: new_array
            },
            ctx: ctx
        }
    }

    pub fn aggregate_from_stack(num_entries: usize, ctx: &'ctx PikeContext)
    -> Self {
        let new_array = unsafe { aggregate_array(num_entries as i32) };
        PikeArray {
            array_ref: PikeArrayRef {
                array: new_array
            },
            ctx: ctx
        }
    }

    pub fn append(&mut self, value: PikeThing) {
        let mut sval: svalue = value.into();
        let new_ptr;
        {
            let old_ref = &self.array_ref;
            new_ptr = unsafe {
                append_array(old_ref.array, &mut sval)
            };
        }
        self.array_ref = PikeArrayRef { array: new_ptr };
    }

    pub fn len(&self) -> usize {
        unsafe {
            (*self.array_ref.array).size as usize
        }
    }
}

pub struct PikeArrayIterator<'ctx> {
  iterator: PikeObject<'ctx, ()>
}

impl<'ctx> Iterator for PikeArrayIterator<'ctx> {
    type Item = (PikeThing);

    fn next(&mut self) -> Option<Self::Item> {
        let ended = self.iterator.call_func("`!", vec![])
            .expect("Error calling \"`!\" in iterator");
        match ended {
            PikeThing::Int(i) => {
                if i.integer != 0 {
                    return None;
                }
            }
            _ => panic!("Wrong type from iterator->`!")
        }

        let val = self.iterator.call_func("value", vec![])
            .expect("Error calling \"value\" in iterator");
        self.iterator.call_func("next", vec![])
            .expect("Error calling \"next\" in iterator");

        Some(val)
    }
}

impl<'ctx> IntoIterator for PikeArray<'ctx> {
    type Item = PikeThing;
    type IntoIter = PikeArrayIterator<'ctx>;

    fn into_iter(self) -> Self::IntoIter {
        let ctx = self.ctx;
        let thing = PikeThing::Array(self.array_ref);
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeArrayIterator::<'ctx> { iterator: it.unwrap(ctx) }
            }
            _ => panic!("Wrong type returned from f_get_iterator")
        }
    }
}

impl<'a> IntoIterator for &'a PikeArray<'a> {
    type Item = PikeThing;
    type IntoIter = PikeArrayIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let ctx = self.ctx;
        let thing = PikeThing::Array(self.array_ref.clone(ctx));
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeArrayIterator { iterator: it.unwrap(ctx) }
            }
            _ => panic!("Wrong type returned from f_get_iterator")
        }
    }
}

impl<'a> Serialize for PikeArray<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for v in self {
            seq.serialize_element(&v.unwrap(self.ctx))?;
        }
        seq.end()
    }
}
