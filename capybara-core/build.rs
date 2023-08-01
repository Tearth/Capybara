use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        windows: { all(target_os = "windows", target_arch = "x86_64") },
        unix: { all(target_os = "linux", target_arch = "x86_64") },
        web: { all(target_os = "unknown", target_arch = "wasm32") },
    }
}
