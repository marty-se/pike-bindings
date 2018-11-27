use ::types::type_deps::*;
use ::serde::ser::*;
use ::ffi::*;

#[derive(Debug)]
pub struct PikeArrayRef {
    ptr: *mut array
}

refcounted_type!(PikeArrayRef, array, DeferredArrayDrop);

struct DeferredArrayDrop {
    ptr: *mut array
}

impl DropWithContext for DeferredArrayDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
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
    /// Returns an empty array with a pre-allocated capacity (but 0 size).
    pub fn with_capacity(capacity: usize, ctx: &'ctx PikeContext) -> Self {
        let array_ref = unsafe {
            PikeArrayRef::from_ptr(real_allocate_array(0, capacity as isize))
        };
        PikeArray { array_ref, ctx }
    }

    /// Returns an array with the specified size.
    pub fn with_size(size: usize, ctx: &'ctx PikeContext) -> Self {
        let array_ref = unsafe {
            PikeArrayRef::from_ptr(real_allocate_array(size as isize, 0))
        };
        PikeArray { array_ref, ctx }
    }

    pub fn aggregate_from_stack(num_entries: usize, ctx: &'ctx PikeContext)
    -> Self {
        let array_ref = unsafe {
            PikeArrayRef::from_ptr(aggregate_array(num_entries as i32))
        };
        PikeArray { array_ref, ctx }
    }

    pub fn append(&mut self, value: PikeThing) {
        let mut sval: svalue = value.into();
        let new_ptr;
        {
            let old_ref = &self.array_ref;
            new_ptr = unsafe {
                append_array(old_ref.as_mut_ptr(), &mut sval)
            };
        }
        self.array_ref = unsafe { PikeArrayRef::from_ptr(new_ptr) };
    }

    pub fn len(&self) -> usize {
        unsafe {
            (*self.array_ref.as_mut_ptr()).size as usize
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
                PikeArrayIterator::<'ctx> {
                    iterator: it.into_with_ctx(ctx)
                }
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
        let thing = PikeThing::Array(self.array_ref.clone_with_ctx(ctx));
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeArrayIterator { iterator: it.into_with_ctx(ctx) }
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
