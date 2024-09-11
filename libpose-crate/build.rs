
fn main() {
    // Where to find the libpose library?
    println!("cargo::rustc-link-lib=dylib=pose");
    println!("cargo:rustc-link-search=native=../libpose/build");
}