use crate::app::ApplicationState;
use crate::window::InputEvent;
use anyhow::Result;
use egui::{Context, FullOutput, RawInput};

pub trait UserSpace {
    fn input(&mut self, state: ApplicationState, event: InputEvent) -> Result<()>;
    fn frame(&mut self, state: ApplicationState, delta: f32) -> Result<()>;
    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<FullOutput>;
}
