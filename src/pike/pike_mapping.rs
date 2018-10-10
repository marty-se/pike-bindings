use ::pike::*;
use ::serde::ser::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{mapping, really_free_mapping, debug_allocate_mapping,
    mapping_insert, f_aggregate_mapping, f_get_iterator, svalue};

// Raw pointers (e.g. *mut mapping) are not Send-safe by default.
// However, we know that Pike won't free the mapping, leaving the pointer
// dangling, as long as we don't decrement the refcount we incremented in
// ::new().
unsafe impl Send for PikeMappingRef {}
unsafe impl Send for DeferredMappingDrop {}

#[derive(Debug)]
pub struct PikeMappingRef {
    mapping: *mut mapping
}

impl PikeMappingRef {
    pub fn new(mapping: *mut mapping, _ctx: &PikeContext) -> Self {
        unsafe {
            (*mapping).refs += 1;
        }
        Self { mapping: mapping }
    }

    pub fn new_without_ref(mapping: *mut mapping) -> Self {
        Self { mapping: mapping }
    }

    // Cannot implement regular Clone trait since we need a &PikeContext
    // argument.
    pub fn clone(&self, ctx: &PikeContext) -> Self {
        Self::new(self.mapping, ctx)
    }

    pub fn unwrap<'ctx>(self, ctx: &'ctx PikeContext) -> PikeMapping<'ctx> {
        PikeMapping { mapping_ref: self, ctx: ctx }
    }

    pub fn as_mut_ptr(&self) -> *mut mapping {
        self.mapping
    }
}

impl Drop for PikeMappingRef {
    fn drop(&mut self) {
        // This may be called anywhere, but we may only decrease refs (and
        // potentially free) while the interpreter lock is held. However, we
        // don't want to acquire the lock here, both for performance reasons and
        // potential locking order predictability issues. Instead, we'll
        // transfer the pointer to a new struct that we send to
        // drop_with_context().
        // Note: our contribution to the reference counter of the raw
        // mapping struct is transferred to the DeferredMappingDrop here.
        let new_ref = DeferredMappingDrop { mapping: self.mapping };
        ::pike::interpreter::drop_with_context(new_ref);
    }
}

struct DeferredMappingDrop {
    mapping: *mut mapping
}

impl DropWithContext for DeferredMappingDrop {
    fn drop_with_context(&self, ctx: &PikeContext) {
        let ptr = self.mapping;
        unsafe {
            (*ptr).refs -= 1;
            if (*ptr).refs == 0 {
                really_free_mapping(ptr);
            }
        }
    }
}

#[derive(Debug)]
pub struct PikeMapping<'a> {
    mapping_ref: PikeMappingRef,
    ctx: &'a PikeContext
}

define_from_impls!(PikeMappingRef, PikeMapping, Mapping, mapping_ref);

impl<'ctx> PikeMapping<'ctx> {
    pub fn from_ref(mapping_ref: PikeMappingRef, ctx: &'ctx PikeContext) -> Self {
        Self { mapping_ref: mapping_ref, ctx: ctx }
    }

    pub fn with_capacity(size: usize, ctx: &'ctx PikeContext) -> Self {
        let new_mapping = unsafe { debug_allocate_mapping(size as i32) };
        PikeMapping {
            mapping_ref: PikeMappingRef {
                mapping: new_mapping
            },
            ctx: ctx
        }
    }

    pub fn insert (&self, key: PikeThing, val: PikeThing) {
        let key_sval: svalue = key.into();
        let val_sval: svalue = val.into();
        unsafe {
            mapping_insert (self.mapping_ref.mapping, &key_sval, &val_sval);
        }
    }

    pub fn aggregate_from_stack(
        num_entries: usize,
        ctx: &'ctx PikeContext) -> Self {
        unsafe {
            // Aggregates a mapping and pushes it to the Pike stack.
            f_aggregate_mapping(num_entries as i32);
        }
        let res_thing = ctx.pop_from_stack();
        match res_thing {
            PikeThing::Mapping(m) => { Self::from_ref(m, ctx) },
            _ => { panic!("Wrong type returned from f_aggregate_mapping"); }
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            (*(*self.mapping_ref.mapping).data).size as usize
        }
    }
}

pub struct PikeMappingIterator<'ctx> {
    iterator: PikeObject<'ctx, ()>
}

impl<'ctx> Iterator for PikeMappingIterator<'ctx> {
  type Item = (PikeThing, PikeThing);

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

    let idx = self.iterator.call_func("index", vec![])
        .expect("Error calling \"index\" in iterator");
    let val = self.iterator.call_func("value", vec![])
        .expect("Error calling \"value\" in iterator");
    self.iterator.call_func("next", vec![])
        .expect("Error calling \"next\" in iterator");;

    Some((idx, val))
  }
}

impl<'ctx> IntoIterator for PikeMapping<'ctx> {
  type Item = (PikeThing, PikeThing);
  type IntoIter = PikeMappingIterator<'ctx>;

  fn into_iter(self) -> Self::IntoIter {
    let ctx = self.ctx;
    let thing = PikeThing::Mapping(self.mapping_ref);
    ctx.push_to_stack(thing);
    unsafe { f_get_iterator(1); }
    match ctx.pop_from_stack() {
      PikeThing::Object(it) => {
        PikeMappingIterator::<'ctx> { iterator: it.unwrap(ctx) }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl<'a> IntoIterator for &'a PikeMapping<'a> {
    type Item = (PikeThing, PikeThing);
    type IntoIter = PikeMappingIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let ctx = self.ctx;
        let thing = PikeThing::Mapping(self.mapping_ref.clone(ctx));
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeMappingIterator { iterator: it.unwrap(ctx) }
            }
            _ => panic!("Wrong type returned from f_get_iterator")
        }
    }
}

impl<'a> Serialize for PikeMapping<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(&k.unwrap(self.ctx), &v.unwrap(self.ctx))?;
        }
        map.end()
    }
}