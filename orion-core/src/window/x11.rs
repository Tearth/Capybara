use super::*;
use ::x11::xlib;
use anyhow::{bail, Result};
use log::Level;
use std::collections::VecDeque;
use std::ptr;

pub struct WindowContext {
    pub size: Coordinates,
    pub cursor_position: Coordinates,
    pub cursor_in_window: bool,

    event_queue: VecDeque<InputEvent>,
}

impl WindowContext {
    pub fn new(title: &str) -> Result<Box<Self>> {
        simple_logger::init_with_level(Level::Debug)?;

        unsafe {
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                return bail!("Error while creating a new display".to_string());
            }

            Ok(Box::new(Self { size: Coordinates::new(800, 600), cursor_position: Default::default(), cursor_in_window: false, event_queue: Default::default() }))
        }
    }

    pub fn poll_event(&mut self) -> Option<InputEvent> {
        None
    }
}
