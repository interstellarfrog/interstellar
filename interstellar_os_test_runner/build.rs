use std::env::var;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rustc-env=OUT_DIR={}", var("OUT_DIR").unwrap());
}
