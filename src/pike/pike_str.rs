use pike::*;
use serde::ser::*;

pub struct PikeString {
  pike_string: *mut pike_string
}

def_pike_type!(PikeString, pike_string, string, PIKE_T_STRING, really_free_string);

impl PikeString {
    pub fn as_ptr(&self) -> *mut pike_string {
        self.pike_string
    }
}

impl From<PikeString> for String {
    fn from(pikestr: PikeString) -> String {
        let pt = PikeThing::PikeString(pikestr);
        pt.push_to_stack();

        unsafe {
            f_string_to_utf8(1);
            let thing = PikeThing::pop_from_stack();
            match thing {
                PikeThing::PikeString(res) => {
                    let pikestr = res.pike_string;
                    let slice: &[i8] =
                        ::std::slice::from_raw_parts(&((*pikestr).str[0]), (*pikestr).len as usize);
                    let slice2: &[u8] = ::std::mem::transmute(slice);
                    let mut v: ::std::vec::Vec<u8> = ::std::vec::Vec::new();
                    v.extend_from_slice(slice2);
                    ::std::string::String::from_utf8(v).unwrap()
                }
                _ => {
                    panic!("string_to_utf8 returned wrong type");
                }
            }
        }
    }
}

impl<'a> From<&'a PikeString> for String {
    fn from(pikestr: &PikeString) -> String {
        let pt = PikeThing::PikeString(pikestr.clone());
        pt.push_to_stack();

        unsafe {
            f_string_to_utf8(1);
            let thing = PikeThing::pop_from_stack();
            match thing {
                PikeThing::PikeString(res) => {
                    let pikestr = res.pike_string;
                    let slice: &[i8] =
                        ::std::slice::from_raw_parts(&((*pikestr).str[0]), (*pikestr).len as usize);
                    let slice2: &[u8] = ::std::mem::transmute(slice);
                    let mut v: ::std::vec::Vec<u8> = ::std::vec::Vec::new();
                    v.extend_from_slice(slice2);
                    ::std::string::String::from_utf8(v).unwrap()
                }
                _ => {
                    panic!("string_to_utf8 returned wrong type");
                }
            }
        }
    }
}

impl From<String> for PikeString {
    fn from(s: String) -> PikeString {
        unsafe {
            let pikestr = PikeString::new(debug_make_shared_binary_string(
                s.as_ptr() as *const i8,
                s.len(),
            ));
            let pt = PikeThing::PikeString(pikestr);
            pt.push_to_stack();

            f_utf8_to_string(1);
            let thing = PikeThing::pop_from_stack();
            match thing {
                PikeThing::PikeString(pikestr) => pikestr,
                _ => {
                    panic!("string_to_utf8 returned wrong type");
                }
            }
        }
    }
}

impl<'a> From<&'a str> for PikeString {
    fn from(s: &str) -> PikeString {
        unsafe {
            let pikestr = PikeString::new(debug_make_shared_binary_string(
                s.as_ptr() as *const i8,
                s.len(),
            ));
            let pt = PikeThing::PikeString(pikestr);
            pt.push_to_stack();

            f_utf8_to_string(1);
            let thing = PikeThing::pop_from_stack();
            match thing {
                PikeThing::PikeString(pikestr) => pikestr,
                _ => {
                    panic!("string_to_utf8 returned wrong type");
                }
            }
        }
    }
}

impl From<Vec<u8>> for PikeString {
    fn from(v: Vec<u8>) -> PikeString {
        unsafe {
            PikeString::new(debug_make_shared_binary_string(
                v.as_ptr() as *const i8,
                v.len(),
            ))
        }
    }
}

impl<'a> From<&'a [u8]> for PikeString {
    fn from(v: &'a [u8]) -> PikeString {
        unsafe {
            PikeString::new(debug_make_shared_binary_string(
                v.as_ptr() as *const i8,
                v.len(),
            ))
        }
    }
}

impl Serialize for PikeString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: String = self.into();
        serializer.serialize_str(&s)
    }
}
