#[cfg(any(windows, unix))]
pub mod native;
#[cfg(any(windows, unix))]
pub type FileSystem = native::FileSystem;

#[cfg(web)]
pub mod web;
#[cfg(web)]
pub type FileSystem = web::FileSystem;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FileLoadingStatus {
    Idle,
    Loading,
    Finished,
}
