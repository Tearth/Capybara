use anyhow::Result;
use cfg_aliases::cfg_aliases;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    let profile = env::var("PROFILE")?;
    let target = env::var("TARGET")?;

    if profile == "release" && target == "x86_64-pc-windows-msvc" {
        println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");
        println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
    }

    if Path::new("./data/boot.zip").exists() {
        fs::remove_file("./data/boot.zip")?;
    }

    if Path::new("./data/main.zip").exists() {
        fs::remove_file("./data/main.zip")?;
    }

    if Path::new("./assets/boot/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/boot.zip", "./assets/boot/*"]).spawn()?.wait()?;
    }

    if Path::new("./assets/main/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/main.zip", "./assets/main/*"]).spawn()?.wait()?;
    }

    if target == "x86_64-pc-windows-msvc" {
        Command::new("llvm-rc").arg("./resources.rc").spawn()?.wait()?;
        println!("cargo:rustc-link-arg=./template/resources.res");
    }

    println!("cargo:rerun-if-changed=./assets/");

    cfg_aliases! {
        windows: { all(target_os = "windows", target_arch = "x86_64") },
        unix: { all(target_os = "linux", target_arch = "x86_64") },
        web: { all(target_os = "unknown", target_arch = "wasm32") },
    }

    Ok(())
}
