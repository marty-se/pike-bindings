use ::bindings::*;
use ::pike::*;
use ::serde::ser::*;

#[derive(Debug)]
pub struct PikeArray {
  array: *mut array
}

def_pike_type!(PikeArray, array, array, PIKE_T_ARRAY, really_free_array);

impl PikeArray {
    /// Returns an empty array with a pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        unsafe {
            PikeArray { array: real_allocate_array(0, capacity as isize) }
        }
    }

    /// Returns an array with the specified size.
    pub fn with_size(size: usize) -> Self {
        unsafe {
            PikeArray { array: real_allocate_array(size as isize, 0) }
        }
    }

    pub fn aggregate_from_stack(num_entries: usize) -> Self {
        unsafe {
            PikeArray { array: aggregate_array(num_entries as i32) }
        }
    }

    pub fn append(&mut self, value: &PikeThing) {
        let mut sval: svalue = value.into();
        unsafe {
            // FIXME: Handle refcounts
            self.array = append_array(self.array, &mut sval);
        }
    }

    pub fn len(&self) -> usize {
        unsafe {
            (*self.array).size as usize
        }
    }
}

pub struct PikeArrayIterator {
  iterator: PikeObject<()>
}

impl Iterator for PikeArrayIterator {
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
