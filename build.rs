extern crate bindgen;

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Tell cargo to tell rustc to link the system bzip2
    // shared library.
//    println!("cargo:rustc-link-lib=bz2");

    let pike_includes_output = Command::new("pike")
        .arg("-x")
        .arg("module")
        .arg("--query=include_path")
        .output()
        .expect("Could not get pike include path");
    let mut pike_includes_str = String::from_utf8(pike_includes_output.stdout).unwrap();
    pike_includes_str.pop(); // Remove newline.

    let pike_includes_path = PathBuf::from(pike_includes_str);
 //   let array_h_path = pike_includes_path.join("array.h");
 
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.

    /*
            .whitelist_recursively(false)
        .whitelist_type("array")
        .whitelist_type("svalue")
        .whitelist_type("anything")
        .whitelist_type("callable")
        .whitelist_type("mapping")
        .whitelist_type("multiset")
        .whitelist_type("object")
        .whitelist_type("program")
        .whitelist_type("pike_string")
        .whitelist_type("pike_type")
        .whitelist_type("ref_dummy")
    */

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        /*
        .whitelist_recursively(false)
        .whitelist_type("array")

        .whitelist_type("svalue")
        .whitelist_type("ref_dummy")
        .whitelist_type("anything")
        .whitelist_type("node")
        .whitelist_type("node_s")
        .whitelist_type("TYPEOF")
        .whitelist_type("SUBTYPEOF")

        .whitelist_type("callable")

        .whitelist_type("mapping")
        .whitelist_type("mapping_data")
        .whitelist_type("keypair")

        .whitelist_type("multiset")
        .whitelist_type("multiset_data")
        .whitelist_type("msnode")
        .whitelist_type("msnode_ind")
        .whitelist_type("msnode_indval")

        .whitelist_type("object")

        .whitelist_type("pike_string")
        .whitelist_type("size_shift")

        .whitelist_type("program")
        .whitelist_type("pike_type")
        .whitelist_type("identifier")
        .whitelist_type("reference")
        .whitelist_type("inherit")
        .whitelist_type("idptr")
        .whitelist_type("program_constant")

        .whitelist_type("visit_enter_cb")
        .whitelist_type("visit_ref_cb")
        .whitelist_type("visit_leave_cb")

        .whitelist_function("visit_array")
        .whitelist_function("visit_mapping")
        .whitelist_function("visit_multiset")
        .whitelist_function("visit_object")
        .whitelist_function("visit_program")
        .whitelist_function("visit_string")
        .whitelist_function("visit_type")

        // Must be called with an svalue.
        .whitelist_function("visit_function")
        .whitelist_function("real_visit_svalues")

        .whitelist_var("VISIT_.*")
        .whitelist_function("type_from_visit_fn")

        .whitelist_type("node_data")
        .whitelist_type("compiler_frame")
        .whitelist_type("node_identifier")
        .whitelist_type("local_variable")

        .whitelist_type("timeval")
        .whitelist_type("__time_t")
        .whitelist_type("__suseconds_t")
*/
        .header(pike_includes_path.join("array.h").to_str().unwrap())
        .header(pike_includes_path.join("svalue.h").to_str().unwrap())
        .header(pike_includes_path.join("mapping.h").to_str().unwrap())
        .header(pike_includes_path.join("multiset.h").to_str().unwrap())
        .header(pike_includes_path.join("object.h").to_str().unwrap())
        .header(pike_includes_path.join("program.h").to_str().unwrap())
        .header(pike_includes_path.join("stralloc.h").to_str().unwrap())
        .header(pike_includes_path.join("multiset.h").to_str().unwrap())
        .header(pike_includes_path.join("interpret.h").to_str().unwrap())
        .header(pike_includes_path.join("las.h").to_str().unwrap())
        .header(pike_includes_path.join("gc.h").to_str().unwrap())
 
         // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");
        
}