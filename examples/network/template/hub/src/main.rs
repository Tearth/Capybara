#[cfg(not(web))]
pub mod core;

#[cfg(not(web))]
pub mod lobby;

#[cfg(not(web))]
pub mod terminal;

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
