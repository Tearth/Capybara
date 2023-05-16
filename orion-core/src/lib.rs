#![allow(
    clippy::while_immutable_condition,
    clippy::never_loop,
    clippy::not_unsafe_ptr_arg_deref,
    clippy::type_complexity,
    clippy::identity_op,
    clippy::too_many_arguments
)]

pub mod app;
pub mod assets;
pub mod filesystem;
pub mod renderer;
pub mod ui;
pub mod user;
pub mod utils;
pub mod window;

pub use anyhow;
pub use egui;
pub use glam;
pub use log;
