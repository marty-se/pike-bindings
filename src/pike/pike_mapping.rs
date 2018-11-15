use ::pike::*;
use ::serde::ser::*;
use ::pike::interpreter::DropWithContext;
use ::ffi::{mapping, really_free_mapping, debug_allocate_mapping,
    mapping_insert, f_aggregate_mapping, f_get_iterator, svalue};

#[derive(Debug)]
pub struct PikeMappingRef {
    ptr: *mut mapping
}

refcounted_type!(PikeMappingRef, mapping, DeferredMappingDrop);

struct DeferredMappingDrop {
    ptr: *mut mapping
}

impl DropWithContext for DeferredMappingDrop {
    fn drop_with_context(&self, _ctx: &PikeContext) {
        let ptr = self.ptr;
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
        let mapping_ref = unsafe {
            PikeMappingRef::from_ptr(debug_allocate_mapping(size as i32))
        };
        PikeMapping { mapping_ref, ctx }
    }

    pub fn insert (&self, key: PikeThing, val: PikeThing) {
        let key_sval: svalue = key.into();
        let val_sval: svalue = val.into();
        unsafe {
            mapping_insert (self.mapping_ref.ptr, &key_sval, &val_sval);
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
            (*(*self.mapping_ref.ptr).data).size as usize
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
                PikeMappingIterator::<'ctx> { iterator: it.into_with_ctx(ctx) }
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
        let thing = PikeThing::Mapping(self.mapping_ref.clone_with_ctx(ctx));
        ctx.push_to_stack(thing);
        unsafe { f_get_iterator(1); }
        match ctx.pop_from_stack() {
            PikeThing::Object(it) => {
                PikeMappingIterator { iterator: it.into_with_ctx(ctx) }
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
