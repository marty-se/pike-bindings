use ::pike::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{multiset, multiset_sizeof, f_get_iterator, really_free_multiset};
use ::serde::ser::*;

// Raw pointers (e.g. *mut multiset) are not Send-safe by default.
// However, we know that Pike won't free the multiset, leaving the pointer
// dangling, as long as we don't decrement the refcount we incremented in
// ::new().
unsafe impl Send for PikeMultisetRef {}
unsafe impl Send for DeferredMultisetDrop {}

#[derive(Debug)]
pub struct PikeMultisetRef {
    multiset: *mut multiset
}

impl PikeMultisetRef {
    pub fn new(multiset: *mut multiset, _ctx: &PikeContext) -> Self {
        unsafe {
            (*multiset).refs += 1;
        }
        Self { multiset: multiset }
    }

    pub fn new_without_ref(multiset: *mut multiset) -> Self {
        Self { multiset: multiset }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.multiset, ctx)
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) -> PikeMultiset<'ctx> {
        PikeMultiset { multiset_ref: self, ctx: ctx }
    }

    pub fn as_mut_ptr(&self) -> *mut multiset {
        self.multiset
    }
}

impl Drop for PikeMultisetRef {
    fn drop(&mut self) {
        // This may be called anywhere, but we may only decrease refs (and
        // potentially free) while the interpreter lock is held. However, we
        // don't want to acquire the lock here, both for performance reasons and
        // potential locking order predictability issues. Instead, we'll
        // transfer the pointer to a new struct that we send to
        // drop_with_context().
        // Note: our contribution to the reference counter of the raw
        // multiset struct is transferred to the DeferredMultisetDrop here.
        let new_ref = DeferredMultisetDrop { multiset: self.multiset };
        ::pike::interpreter::drop_with_context(new_ref);
    }
}

struct DeferredMultisetDrop {
    multiset: *mut multiset
}

impl DropWithContext for DeferredMultisetDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.multiset;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                really_free_multiset(ptr);
            }
        }
    }
}

#[derive(Debug)]
pub struct PikeMultiset<'ctx> {
    multiset_ref: PikeMultisetRef,
    ctx: &'ctx PikeContext
}

impl<'ctx> PikeMultiset<'ctx> {
    pub fn len(&self) -> usize {
        unsafe {
            multiset_sizeof(self.multiset_ref.multiset) as usize
        }
    }
}

pub struct PikeMultisetIterator<'ctx> {
    iterator: PikeObject<'ctx, ()>
}

impl<'ctx> Iterator for PikeMultisetIterator<'ctx> {
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

        let val = self.iterator.call_func("index", vec![])
            .expect("Error calling \"index\" in iterator");
        self.iterator.call_func("next", vec![])
            .expect("Error calling \"next\" in iterator");

        Some(val)
    }
}

impl<'ctx> IntoIterator for PikeMultiset<'ctx> {
    type Item = PikeThing;
    type IntoIter = PikeMultisetIterator<'ctx>;

    fn into_iter(self) -> Self::IntoIter {
        let ctx = self.ctx;
        let thing = PikeThing::Multiset(self.multiset_ref);
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeMultisetIterator { iterator: it.unwrap(ctx) }
            }
            _ => panic!("Wrong type returned from f_get_iterator")
        }
    }
}

impl<'a> IntoIterator for &'a PikeMultiset<'a> {
    type Item = PikeThing;
    type IntoIter = PikeMultisetIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let ctx = self.ctx;
        let thing = PikeThing::Multiset(self.multiset_ref.clone(ctx));
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeMultisetIterator { iterator: it.unwrap(ctx) }
            }
            _ => panic!("Wrong type returned from f_get_iterator")
        }
    }
}

impl<'a> Serialize for PikeMultiset<'a> {
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