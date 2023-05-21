use orion_core::assets::bundler;
use std::path::Path;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target = std::env::var("TARGET").unwrap();

    if profile == "release" && target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
        println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }

    if Path::new("./assets/boot/").exists() {
        bundler::pack("./assets/boot/", "./data/boot.zip").unwrap();
        println!("cargo:rerun-if-changed=./assets/boot/");
    }

    if Path::new("./assets/main/").exists() {
        bundler::pack("./assets/main/", "./data/main.zip").unwrap();
        println!("cargo:rerun-if-changed=./assets/main/");
    }
}
