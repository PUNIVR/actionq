use std::env;

fn main() {

    let out_dir = env::var("OUT_DIR").unwrap();

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

    // Link jetson-inference and jetson-utils
    println!("cargo:rustc-link-search=native=/home/nvidia/PREPARE/prepose/libs");
    println!("cargo:rustc-link-lib=dylib=jetson-inference");
    println!("cargo:rustc-link-lib=dylib=jetson-utils");

    // Link c++ stdlib
    println!("cargo:rustc-link-lib=dylib=stdc++");
}