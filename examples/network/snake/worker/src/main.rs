#![allow(clippy::single_match, clippy::collapsible_else_if, clippy::await_holding_lock, clippy::collapsible_if)]

#[cfg(not(web))]
pub mod core;

#[cfg(not(web))]
pub mod terminal;

#[cfg(not(web))]
pub mod config;

#[cfg(not(web))]
pub mod room;

fn main() {
    #[cfg(not(web))]
    internal::main();
}

#[cfg(not(web))]
mod internal {
    use super::core::Core;

    #[tokio::main]
    pub async fn main() {
        Core::new().run().await;
    }
}
