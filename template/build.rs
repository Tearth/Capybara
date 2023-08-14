use cfg_aliases::cfg_aliases;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let profile = env::var("PROFILE").unwrap();
    let target = env::var("TARGET").unwrap();

    if profile == "release" && target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
        println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }

    if Path::new("./assets/boot/").exists() {
        fs::remove_file("./data/boot.zip").unwrap();
        Command::new("7z").args(["a", "-tzip", "./data/boot.zip", "./assets/boot/*"]).spawn().unwrap();
        println!("cargo:rerun-if-changed=./assets/boot/");
    }

    if Path::new("./assets/main/").exists() {
        fs::remove_file("./data/main.zip").unwrap();
        Command::new("7z").args(["a", "-tzip", "./data/main.zip", "./assets/main/*"]).spawn().unwrap();
        println!("cargo:rerun-if-changed=./assets/main/");
    }

    cfg_aliases! {
        windows: { all(target_os = "windows", target_arch = "x86_64") },
        unix: { all(target_os = "linux", target_arch = "x86_64") },
        web: { all(target_os = "unknown", target_arch = "wasm32") },
    }
}
