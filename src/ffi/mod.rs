#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/pike-ffi.rs"));

impl std::fmt::Debug for array {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("array")
            .field("refs", &self.refs)
            .finish()
    }
}
