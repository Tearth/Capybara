use anyhow::Result;
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

    if Path::new("./data/data0.zip").exists() {
        fs::remove_file("./data/data0.zip")?;
    }

    if Path::new("./assets/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/data0.zip", "./assets/*"]).spawn()?;
    }

    println!("cargo:rerun-if-changed=./assets/");

    Ok(())
}
