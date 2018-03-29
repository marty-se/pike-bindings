#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern {
  static mut visit_ref: *const visit_ref_cb;
}

#[no_mangle]
pub unsafe extern "C" fn process_thing(dst_thing: *mut::std::os::raw::c_void,
                                       ref_type: ::std::os::raw::c_int,
                                       visit_dst: visit_thing_fn,
                                       extra : * mut::std::os::raw::c_void)
{

}

#[no_mangle]
pub extern fn process_svalue(s: *const svalue)
{
    unsafe {
        visit_ref = &Some(process_thing);
    }
}

#[no_mangle]
pub extern fn pike_module_init() -> () {
    println!("Hello?");
}

#[no_mangle]
pub extern fn pike_module_exit() -> () {

}
