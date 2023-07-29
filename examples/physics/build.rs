use std::path::Path;
use std::process::Command;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target = std::env::var("TARGET").unwrap();

    if profile == "release" && target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
        println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }

    if Path::new("./assets/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/data0.zip", "./assets/*"]).spawn().unwrap();
        println!("cargo:rerun-if-changed=./assets/");
    }
}
