fn main() {
    // Set bridge file for Rust <-> C++ comunication
    let mut cc = cxx_build::bridge("src/zed.rs");

    // Setup C++
    let cc = cc.std("c++17");

    // Link and compile
    cc.file("src/zed.cc")
        .flag("-w") // Disable annoying warnings!
        .includes(&[
            "/usr/local/zed/include/",
            "/usr/local/cuda-11.4/include",
            "include",
        ])
        .compile("actionq-zed-cxx.a");

    // Link CUDA runtime (libcudart.so)
    println!("cargo:rustc-link-search=native=/usr/local/cuda/lib64");
    println!("cargo:rustc-link-lib=cudart");

    // Link ZED SDK
    println!("cargo:rustc-link-search=native=/usr/local/zed/lib");
    //println!("cargo:rustc-link-lib=sl_zed_static"); 
    println!("cargo:rustc-link-lib=sl_ai");
    println!("cargo:rustc-link-lib=sl_zed");

    println!("cargo:rerun-if-changed=src/zed.cc");
    println!("cargo:rerun-if-changed=include/zed.hh");
}
