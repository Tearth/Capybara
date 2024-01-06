use crate::app::ApplicationState;
use crate::window::InputEvent;
use anyhow::Result;
use egui::FullOutput;
use egui::RawInput;

#[derive(Clone, Debug, PartialEq)]
pub enum FrameCommand {
    ChangeScene { name: String },
    ResetScene,
    Exit,
}

pub trait Scene<G> {
    fn activation(&mut self, state: ApplicationState<G>) -> Result<()>;
    fn deactivation(&mut self, state: ApplicationState<G>) -> Result<()>;

    fn input(&mut self, state: ApplicationState<G>, event: InputEvent) -> Result<()>;
    fn fixed(&mut self, state: ApplicationState<G>) -> Result<Option<FrameCommand>>;
    fn frame(&mut self, state: ApplicationState<G>, accumulator: f32, delta: f32) -> Result<Option<FrameCommand>>;
    fn ui(&mut self, state: ApplicationState<G>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)>;

    fn reset(&self) -> Box<dyn Scene<G>>;
}
