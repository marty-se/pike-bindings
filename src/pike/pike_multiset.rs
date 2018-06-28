use ::bindings::*;
use ::pike::*;
use ::serde::ser::*;

#[derive(Debug)]
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