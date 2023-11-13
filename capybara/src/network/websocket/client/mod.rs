#[cfg(any(windows, unix))]
pub mod native;
#[cfg(any(windows, unix))]
pub type WebSocketClient = native::WebSocketClient;

#[cfg(web)]
pub mod web;
#[cfg(web)]
pub type WebSocketClient = web::WebSocketClient;
