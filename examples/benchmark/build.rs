fn main() {
    #[cfg(windows)]
    #[cfg(not(debug_assertions))]
    println!("cargo:rustc-link-arg=/EXPORT:NvOptimusEnablement");

    #[cfg(windows)]
    #[cfg(not(debug_assertions))]
    println!("cargo:rustc-link-arg=/EXPORT:AmdPowerXpressRequestHighPerformance");
}
