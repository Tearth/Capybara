use crate::app::ApplicationState;
use crate::window::InputEvent;
use anyhow::Result;
use egui::FullOutput;
use egui::RawInput;

pub enum FrameCommand {
    ChangeScene { name: String },
    Exit,
}

pub trait Scene {
    fn activation(&mut self, state: ApplicationState) -> Result<()>;
    fn deactivation(&mut self, state: ApplicationState) -> Result<()>;

    fn input(&mut self, state: ApplicationState, event: InputEvent) -> Result<()>;
    fn frame(&mut self, state: ApplicationState, delta: f32) -> Result<Option<FrameCommand>>;
    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)>;
}
