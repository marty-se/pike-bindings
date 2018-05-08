use ::bindings::*;
use ::pike::*;
use ::serde::ser::*;

pub struct PikeArray {
  array: *mut array
}

def_pike_type!(PikeArray, array, array, PIKE_T_ARRAY, really_free_array);

impl PikeArray {
  pub fn aggregate_from_stack(num_entries: usize) -> Self {
    unsafe {
      PikeArray { array: aggregate_array(num_entries as i32) }
    }
  }

  pub fn len(&self) -> usize {
    unsafe {
      (*self.array).size as usize
    }
  }
}

pub struct PikeArrayIterator {
  iterator: PikeObject
}

impl Iterator for PikeArrayIterator {
  type Item = (PikeThing);

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

    let val = self.iterator.call_func("value", 0).unwrap();
    self.iterator.call_func("next", 0);

    Some(val)
  }
}

impl IntoIterator for PikeArray {
  type Item = PikeThing;
  type IntoIter = PikeArrayIterator;

  fn into_iter(self) -> Self::IntoIter {
    let thing = PikeThing::Array(self);
    thing.push_to_stack();
    unsafe { f_get_iterator(1); }
    match PikeThing::pop_from_stack() {
      PikeThing::Object(it) => {
        PikeArrayIterator { iterator: it }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl<'a> IntoIterator for &'a PikeArray {
  type Item = PikeThing;
  type IntoIter = PikeArrayIterator;

  fn into_iter(self) -> Self::IntoIter {
    let thing = PikeThing::Array(self.clone());
    thing.push_to_stack();
    unsafe { f_get_iterator(1); }
    match PikeThing::pop_from_stack() {
      PikeThing::Object(it) => {
        PikeArrayIterator { iterator: it }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl Serialize for PikeArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        for v in self {
            seq.serialize_element(&v)?;
        }
        seq.end()
    }
}