use ::types::type_deps::*;
use ::ffi::{multiset, multiset_sizeof, f_get_iterator, really_free_multiset,
    svalue};
use ::serde::ser::*;

#[derive(Debug)]
pub struct PikeMultisetRef {
    ptr: *mut multiset
}

refcounted_type!(PikeMultisetRef, multiset, DeferredMultisetDrop);

struct DeferredMultisetDrop {
    ptr: *mut multiset
}

impl DropWithContext for DeferredMultisetDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
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

define_from_impls!(PikeMultisetRef, PikeMultiset, Multiset, multiset_ref);

impl<'ctx> PikeMultiset<'ctx> {
    pub fn len(&self) -> usize {
        unsafe {
            multiset_sizeof(self.multiset_ref.ptr) as usize
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
                PikeMultisetIterator { iterator: it.into_with_ctx(ctx) }
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
        let thing = PikeThing::Multiset(self.multiset_ref.clone_with_ctx(ctx));
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeMultisetIterator { iterator: it.into_with_ctx(ctx) }
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