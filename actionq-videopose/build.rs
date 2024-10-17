use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

fn dynlink_jetson_libs() {
    println!("cargo:rustc-link-lib=dylib=jetson-inference");
    println!("cargo:rustc-link-lib=dylib=jetson-utils");
}

fn build_libpose() {

    // Rebuild if the pose.cpp file has changes
    println!("cargo::rerun-if-changed=cpp/pose.cpp");

    // Compile artifact libpose.so
    cc::Build::new().cpp(true).flag("-w")
        .cpp_link_stdlib("stdc++").cuda(true)
        .file("cpp/videopose.cpp")
        .includes([
            "/usr/local/cuda/include",
            "extern",
        ])
        .compile("videopose");
}

fn main() {
    dynlink_jetson_libs();
    build_libpose();
}