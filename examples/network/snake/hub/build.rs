use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        web: { target_arch = "wasm32" },
    }
}
