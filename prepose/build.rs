use std::env;

fn main() {

    let out_dir = env::var("OUT_DIR").unwrap();

    // Rebuild library if cpp code has changed
    println!("cargo::rerun-if-changed=cpp/library.cpp");

    // Link jetson-inference and jetson-utils
    println!("cargo:rustc-link-search=native=/home/nvidia/Repositories/prepare-core/prepose/libs");
    println!("cargo:rustc-link-lib=dylib=jetson-inference");
    println!("cargo:rustc-link-lib=dylib=jetson-utils");

    // Link c++ stdlib
    println!("cargo:rustc-link-lib=dylib=stdc++");

    // Compile libpose
    cc::Build::new()
        .cpp(true)
        .file("cpp/library.cpp")
        .include("include")
        .include("/usr/local/cuda/include")
        .compile("libpose.a");

    
    // Link libpose
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=pose");
}