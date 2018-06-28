use ::bindings::*;
use ::pike::*;
use ::serde::ser::*;

pub struct PikeMultiset {
  multiset: *mut multiset
}

def_pike_type!(PikeMultiset, multiset, multiset, PIKE_T_MULTISET, really_free_multiset);

impl PikeMultiset {
  pub fn len(&self) -> usize {
    unsafe {
      multiset_sizeof(self.multiset) as usize
    }
  }
}

pub struct PikeMultisetIterator {
  iterator: PikeObject<()>
}

impl Iterator for PikeMultisetIterator {
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

    let val = self.iterator.call_func("index", 0).unwrap();
    self.iterator.call_func("next", 0);

    Some(val)
  }
}

impl IntoIterator for PikeMultiset {
  type Item = PikeThing;
  type IntoIter = PikeMultisetIterator;

  fn into_iter(self) -> Self::IntoIter {
    let thing = PikeThing::Multiset(self);
    thing.push_to_stack();
    unsafe { f_get_iterator(1); }
    match PikeThing::pop_from_stack() {
      PikeThing::Object(it) => {
        PikeMultisetIterator { iterator: it }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl<'a> IntoIterator for &'a PikeMultiset {
  type Item = PikeThing;
  type IntoIter = PikeMultisetIterator;

  fn into_iter(self) -> Self::IntoIter {
    let thing = PikeThing::Multiset(self.clone());
    thing.push_to_stack();
    unsafe { f_get_iterator(1); }
    match PikeThing::pop_from_stack() {
      PikeThing::Object(it) => {
        PikeMultisetIterator { iterator: it }
      }
      _ => panic!("Wrong type returned from f_get_iterator")
    }
  }
}

impl Serialize for PikeMultiset {
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