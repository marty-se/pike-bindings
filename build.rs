extern crate bindgen;
extern crate encoding;

use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

fn main() {
    generate_pike_bindings();
    generate_sys_bindings();
}

fn generate_sys_bindings()
{
    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/ffi/sys-bindings-wrapper.h")
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("sys-bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn generate_pike_bindings()
{
    let pike_includes_output = Command::new("pike")
        .arg("-x")
        .arg("module")
        .arg("--query=include_path")
        .output()
        .expect("Could not get pike include path");
    let mut pike_includes_str = String::from_utf8(pike_includes_output.stdout).unwrap();
    pike_includes_str.pop(); // Remove newline.

    let pike_includes_path = PathBuf::from(pike_includes_str);

    let mut builder = bindgen::Builder::default()
        .whitelist_recursively(true)

        // Refcounted Pike types can't be copied without refcount handling, so Clone is implemented instead.
        .no_copy("svalue")
        .no_copy("array")
        .no_copy("mapping")
        .no_copy("multiset")
        .no_copy("object")
        .no_copy("pike_string")
        .no_copy("program")
        .no_copy("pike_type")

        .whitelist_function("Pike_error")

        .whitelist_function("init_recovery")
        .whitelist_var("JMP_BUF")
        .whitelist_var("throw_value")

        .whitelist_function("push_text")

        .whitelist_type("svalue")
        .whitelist_var("NUMBER_.*")

        .whitelist_function("aggregate_array")
        .whitelist_function("real_allocate_array")
        .whitelist_function("append_array")

        .whitelist_function("f_aggregate_mapping")
        .whitelist_function("mapping_insert")
        .whitelist_function("debug_allocate_mapping")

        .whitelist_function("f_get_iterator")
        .whitelist_function("multiset_sizeof")

        .whitelist_function("really_free_.*")
        .whitelist_function("schedule_really_free_object")

        .whitelist_function("safe_apply.*")
        .whitelist_function("apply.*")
        .whitelist_function("debug_master")

        .whitelist_var("[a-z]*_type_string")
        .whitelist_function("f_string_to_utf8")
        .whitelist_function("f_utf8_to_string")

        .whitelist_type("visit_thing_fn")
        .whitelist_type("visit_enter_cb")
        .whitelist_type("visit_ref_cb")
        .whitelist_type("visit_leave_cb")
        .whitelist_var("VISIT_.*")
        .whitelist_function("type_from_visit_fn")
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

        .whitelist_function("quick_add_function")
        .whitelist_function("pike_add_function2")
        .whitelist_function("debug_start_new_program")
        .whitelist_function("debug_end_program")
        .whitelist_function("debug_end_class")
        .whitelist_function("pike_set_prog_event_callback")
        .whitelist_function("low_add_storage")
        .whitelist_function("add_program_constant")
        .whitelist_function("debug_clone_object")
        .whitelist_function("program_index_no_free")

        .whitelist_function("pike_threads_allow")
        .whitelist_function("pike_threads_disallow")
        .whitelist_function("call_with_interpreter")

        .whitelist_var("Pike_compiler")

        .whitelist_var("PROG_EVENT_.*")

        .whitelist_function("debug_make_shared_.*")

        .whitelist_var("OPT_.*")
        .whitelist_var("PIKE_T_.*")

        .whitelist_var("Pike_interpreter_pointer");

    let header_fnames = vec!["array.h", "svalue.h", "mapping.h", "multiset.h",
        "object.h", "program.h", "stralloc.h", "multiset.h", "interpret.h", "las.h",
        "gc.h", "global.h", "machine.h", "pike_types.h", "builtin_functions.h",
        "threads.h"];

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    let paths = fs::read_dir(pike_includes_path.to_str().unwrap()).unwrap();

    let in_encoding = encoding::all::ISO_8859_1 as encoding::EncodingRef;
    let in_trap = encoding::DecoderTrap::Ignore;

    // Loop over all .h files in the include directory for preprocessing. However, only the headers
    // listed in header_fnames will be added to the builder (and they may require non-listed headers.)
    for path in paths {
        let fname = path.unwrap().file_name();
        let fname_str = fname.to_str().unwrap();
        if !fname_str.ends_with(".h") {
            continue;
        }
        let joined_out_path = out_path.join(fname_str);

        let joined_incl_path = pike_includes_path.join(fname_str);
        let header_path = joined_incl_path.to_str().unwrap();
        let mut f = File::open(header_path).expect("File not found");
        let mut bytes = Vec::new();
        f.read_to_end(&mut bytes).expect("Could not read file");

        let mut contents = in_encoding.decode(&bytes, in_trap).unwrap();
        if !fname_str.ends_with("global.h") {
            // Bindgen doesn't understand the PMOD_EXPORT prefix, so let's just remove it.
            contents = contents.replace("PMOD_EXPORT ", "");
        }

        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&joined_out_path)
            .expect("Could not create file");
        f.write_all(&contents.into_bytes()).expect("Could not write file");

        if header_fnames.iter().filter(|&header_fname| fname_str.ends_with(header_fname)).next() != None {
            // Only add headers explicitly listed in header_fnames to the list, since some
            // other headers seem to cause issues.
            builder = builder.header(joined_out_path.to_str().unwrap());
        }
    }

    let bindings = builder.generate().expect("Unable to generate bindings");

    let bindings_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    std::fs::create_dir_all(&bindings_dir).expect("Could not create bindings dir");

    bindings.write_to_file(bindings_dir.join("pike-ffi.rs")).expect("Couldn't write bindings!");
}