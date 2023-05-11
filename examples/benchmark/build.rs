use orion_core::assets::bundler;

fn main() {
    #[cfg(not(any(debug_assertions, target_os = "unknown", target_arch = "wasm32")))]
    println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");

    #[cfg(not(any(debug_assertions, target_os = "unknown", target_arch = "wasm32")))]
    println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");

    bundler::pack("./assets/", "./target/assets.zip").unwrap();
    println!("cargo:rerun-if-changed=./assets/");
}
