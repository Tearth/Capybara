use anyhow::bail;
use anyhow::Result;
use cfg_aliases::cfg_aliases;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() -> Result<()> {
    let host = env::var("HOST")?;
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

    if Path::new("./dev/").exists() {
        let name = match host.as_str() {
            "x86_64-pc-windows-msvc" => "free-tex-packer-cli.cmd",
            "x86_64-unknown-linux-gnu" => "free-tex-packer-cli",
            _ => bail!("Invalid target"),
        };

        fs::create_dir_all("./target/tmp/boot/")?;
        fs::create_dir_all("./target/tmp/main/textures/")?;
        fs::create_dir_all("./target/tmp/main/ui/")?;

        Command::new(name).args(["--project", "./textures.ftpp"]).current_dir("./dev").spawn()?.wait()?;
        Command::new(name).args(["--project", "./ui.ftpp"]).current_dir("./dev").spawn()?.wait()?;
    }

    if Path::new("./assets/boot/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/boot.zip", "./assets/boot/*"]).spawn()?.wait()?;
        Command::new("7z").args(["a", "-tzip", "./data/boot.zip", "./target/tmp/boot/*"]).spawn()?.wait()?;
    }

    if Path::new("./assets/main/").exists() {
        Command::new("7z").args(["a", "-tzip", "./data/main.zip", "./assets/main/*"]).spawn()?.wait()?;
        Command::new("7z").args(["a", "-tzip", "./data/main.zip", "./target/tmp/main/*"]).spawn()?.wait()?;
    }

    if target == "x86_64-pc-windows-msvc" {
        Command::new("llvm-rc").arg("./resources.rc").spawn()?.wait()?;
        println!("cargo:rustc-link-arg=./examples/template/resources.res");
    }

    println!("cargo:rerun-if-changed=./assets/");
    println!("cargo:rerun-if-changed=./dev/");

    cfg_aliases! {
        web: { target_arch = "wasm32" },
    }

    Ok(())
}
