use std::{env, path::PathBuf};

fn main() {
    // link to dart_dll/lib/libdart_dll.so
    println!("cargo:rustc-link-search=./dart_dll/lib");
    println!("cargo:rustc-link-lib=dylib=dart_dll");

    // // bindgen with include
    let bindings = bindgen::Builder::default()
        .header("dart_dll/include/dart_dll.h")
        .header("dart_dll/include/dart_api.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // write to bindings.rs
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
