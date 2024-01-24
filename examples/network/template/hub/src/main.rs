#[cfg(not(web))]
pub mod core;

#[cfg(not(web))]
pub mod lobby;

#[cfg(not(web))]
pub mod terminal;

#[cfg(not(web))]
pub mod servers;

#[cfg(not(web))]
pub mod config;

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
