extern crate cmake;
use cmake::Config;
use std::env;

use std::path::PathBuf;

fn main() {
    //mkdir /tmp/MacOS-SDK-include
    //ln -s /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX10.14.sdk/System/Library/Frameworks/CoreFoundation.framework/Headers/ /tmp/MacOS-SDK-include/CoreFoundation

    let bindings = bindgen::Builder::default()
        .header("src/bindings.h")
        .generate_comments(true)
        .use_core()
        //.ctypes_prefix("libc")
        .whitelist_function("CBLDatabase_.*")
        .whitelist_function("CBLDocument_.*")
        .whitelist_function("CBLQuery_.*")
        .whitelist_function("CBLResultSet_.*")
        .whitelist_function("FLValue_GetType")
        .whitelist_function("FLValue_AsString")
        .whitelist_function("FLValue_ToJSON")
        .whitelist_function("FLDict_Get")
        .whitelist_function("FLMutableDict_Set")
        .whitelist_function("FLSlot_SetString")
        .whitelist_function("CBLEndpoint_NewWithURL")
        .whitelist_function("CBLBlob_.*")
        .whitelist_function("CBLReplicator_.*")

        //.whitelist_type("CBLDocument")
        .prepend_enum_name(false)
        //.constified_enum_module("MDB_cursor_op") // allows access to enum values as MDB_cursor_op.MDB_NEXT
        //.derive_debug(true)
        //.impl_debug(true)
        .clang_arg("-I./libCouchbaseLiteC/vendor/couchbase-lite-core/vendor/fleece/API/")
        .clang_arg("-I./libCouchbaseLiteC/include/cbl")
        .clang_arg("-I/tmp/MacOS-SDK-include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from("src");
    bindings.write_to_file(out_path.join("bindings.rs")).expect("Couldn't write bindings!");

    let dst = Config::new("libCouchbaseLiteC").build();
    /*
    For static build

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=CouchbaseLiteCStatic");

    println!("cargo:rustc-link-search=native={}/build/vendor/couchbase-lite-core", dst.display());
    println!("cargo:rustc-link-lib=static=LiteCoreStatic");

    println!("cargo:rustc-link-search=native={}/build/vendor/couchbase-lite-core/vendor/BLIP-Cpp", dst.display());
    println!("cargo:rustc-link-lib=static=BLIPStatic");

    println!("cargo:rustc-link-search=native={}/build/vendor/couchbase-lite-core/vendor/fleece", dst.display());
    println!("cargo:rustc-link-lib=static=FleeceStatic");

    */
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=dylib=CouchbaseLiteC");

    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        // see for exampla : https://github.com/rust-lang/rust/blob/master/src/libstd/build.rs
        println!("cargo:rustc-link-lib=dylib=c++");
        println!("cargo:rustc-link-lib=framework=Foundation");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    } else {
        unimplemented!();
    }
}
