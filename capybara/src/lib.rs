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
pub mod scene;
pub mod ui;
pub mod utils;
pub mod window;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(feature = "physics")]
pub mod physics;

pub use anyhow;
pub use egui;
pub use fastrand;
pub use glam;
pub use instant;
pub use log;
pub use rustc_hash;

#[cfg(feature = "audio")]
pub use kira;

#[cfg(feature = "physics")]
pub use rapier2d;

#[cfg(feature = "physics")]
pub use nalgebra;

#[macro_export]
macro_rules! fast_gpu {
    () => {
        #[no_mangle]
        #[cfg(windows)]
        pub static NvOptimusEnablement: i32 = 1;

        #[no_mangle]
        #[cfg(windows)]
        pub static AmdPowerXpressRequestHighPerformance: i32 = 1;
    };
}

#[macro_export]
macro_rules! error_return {
    ($($arg:tt)+) => { { log::error!($($arg)+); return; } };
}

#[macro_export]
macro_rules! error_break {
    ($($arg:tt)+) => { { log::error!($($arg)+); break; } };
}

#[macro_export]
macro_rules! error_continue {
    ($($arg:tt)+) => { { log::error!($($arg)+); continue; } };
}
