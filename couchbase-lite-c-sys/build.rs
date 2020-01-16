extern crate cmake;
use cmake::Config;
use std::env;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bindgen_common() -> bindgen::Builder {
    bindgen::Builder::default()
    .header("src/bindings.h")
    .generate_comments(true)
    .use_core()
    //.ctypes_prefix("libc")
    .whitelist_function("CBLDatabase_.*")
    .whitelist_function("CBLDocument_.*")
    .whitelist_function("CBL_Release")
    .whitelist_function("CBLQuery_.*")
    .whitelist_function("CBLResultSet_.*")
    .whitelist_function("FLValue_GetType")
    .whitelist_function("FLValue_AsString")
    .whitelist_function("FLValue_ToJSON")
    .whitelist_function("FLDict_Get")
    .whitelist_function("FLDict_IsEmpty")
    .whitelist_function("FLMutableDict_Set")
    .whitelist_function("FLDict_AsMutable")
    .whitelist_function("FLSlot_SetString")
    .whitelist_function("FLSlot_SetNull")
    .whitelist_function("FLSlot_SetData")
    .whitelist_function("FLSlot_SetValue")
    .whitelist_function("FLSlot_SetBool")
    .whitelist_function("FLSlot_SetInt")
    .whitelist_function("FLStr")
    .whitelist_function("CBLEndpoint_NewWithURL")
    .whitelist_function("CBLEndpoint_Free")
    .whitelist_function("CBLBlob_.*")
    .whitelist_function("CBLReplicator_.*")
    .whitelist_function("CBLAuth_.*")
    .prepend_enum_name(false)
    .clang_arg("-I./libCouchbaseLiteC/vendor/couchbase-lite-core/vendor/fleece/API/")
    .clang_arg("-I./libCouchbaseLiteC/include/cbl")
}

fn main_apple() {
    match fs::create_dir_all("/tmp/MacOS-SDK-include") {
        Ok(_) => {}
        Err(e) => panic!("Cannot create directory: {:?}", e),
    };
    let _output = Command::new("/bin/ln")
        .arg("-s")
        .arg("/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk/System/Library/Frameworks/CoreFoundation.framework/Headers/")
        .arg("/tmp/MacOS-SDK-include/CoreFoundation")
        .output()
        .expect("failed to create symbolink link to CoreFoundation Framework");


    let bindings = bindgen_common()
        .clang_arg("-I/tmp/MacOS-SDK-include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src");
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");

    let dst = Config::new("libCouchbaseLiteC").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());

    println!("cargo:rustc-link-lib=dylib=CouchbaseLiteC");

    println!("cargo:rustc-link-lib=dylib=c++");
    println!("cargo:rustc-link-lib=framework=Foundation");
}

fn main_linux() {

    let bindings = bindgen_common()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src");
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");

    let dst = Config::new("libCouchbaseLiteC").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());

    println!("cargo:rustc-link-lib=dylib=CouchbaseLiteC");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

fn main_android() {
    main_linux();
}

fn main_windows() {
    let bindings = bindgen_common()
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src");
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");

    let _dst = Config::new("libCouchbaseLiteC").build();

    let target = env::var("TARGET").unwrap();
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rustc-link-search=native={}/{}", Path::new(&dir).join("libs").display(), target);

    println!("cargo:rustc-link-lib=dylib=CouchbaseLiteC");
    println!("cargo:rustc-link-lib=dylib=stdc++");
}

fn main() {
    if cfg!(target_vendor = "apple") {
        main_apple();
    } else if cfg!(target_os = "linux") {
        main_linux();
    } else if cfg!(target_os = "android") {
        main_android();
    } else if cfg!(target_os = "windows") {
        main_windows();
    }
}
