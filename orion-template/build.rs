use std::path::Path;
use std::process::Command;

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target = std::env::var("TARGET").unwrap();

    if profile == "release" && target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
        println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }

    if Path::new("./assets/boot/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/boot.zip", "./assets/boot/*"]).spawn().unwrap();
        println!("cargo:rerun-if-changed=./assets/boot/");
    }

    if Path::new("./assets/main/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/main.zip", "./assets/main/*"]).spawn().unwrap();
        println!("cargo:rerun-if-changed=./assets/main/");
    }
}
