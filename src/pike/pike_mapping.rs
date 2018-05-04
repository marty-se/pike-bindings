use ::bindings::*;
use ::serde::ser::*;

pub struct PikeMapping {
  mapping: *mut mapping
}

def_pike_type!(PikeMapping, mapping, mapping, PIKE_T_MAPPING, really_free_mapping);

use ::pike::*;

impl PikeMapping {
  pub fn aggregate_from_stack(num_entries: usize) -> Self {
    unsafe {
      f_aggregate_mapping(num_entries as i32); // Aggregates a mapping and pushes it to the Pike stack.
    }
    let res_thing = PikeThing::pop_from_stack();
    match res_thing {
      PikeThing::Mapping(m) => { m },
      _ => { panic!("Wrong type returned from f_aggregate_mapping"); }
    }
  }

  pub fn len(&self) -> usize {
    unsafe {
      (*(*self.mapping).data).size as usize
    }
  }
}

pub struct PikeMappingIterator {
  iterator: PikeObject
}

impl Iterator for PikeMappingIterator {
  type Item = (PikeThing, PikeThing);

  fn next(&mut self) -> Option<Self::Item> {
    let ended = self.iterator.call_func("`!", 0).unwrap();
    match ended {
      PikeThing::Int(i) => {
        if i.integer != 0 {
          return None;
        }
      }
      _ => panic!("Wrong type from iterator->`!")
    }

    let idx = self.iterator.call_func("index", 0).unwrap();
    let val = self.iterator.call_func("value", 0).unwrap();
    self.iterator.call_func("next", 0);

    Some((idx, val))
  }
}

impl IntoIterator for PikeMapping {
  type Item = (PikeThing, PikeThing);
  type IntoIter = PikeMappingIterator;

  fn into_iter(self) -> Self::IntoIter {
    let thing = PikeThing::Mapping(self);
    thing.push_to_stack();
    unsafe { f_get_iterator(1); }
    match PikeThing::pop_from_stack() {
      PikeThing::Object(it) => {
        PikeMappingIterator { iterator: it }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl<'a> IntoIterator for &'a PikeMapping {
  type Item = (PikeThing, PikeThing);
  type IntoIter = PikeMappingIterator;

  fn into_iter(self) -> Self::IntoIter {
    let thing = PikeThing::Mapping(self.clone());
    thing.push_to_stack();
    unsafe { f_get_iterator(1); }
    match PikeThing::pop_from_stack() {
      PikeThing::Object(it) => {
        PikeMappingIterator { iterator: it }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl Serialize for PikeMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}