use crate::app::ApplicationState;
use crate::window::InputEvent;
use egui::Context;

pub trait UserSpace {
    fn input(&mut self, state: ApplicationState, event: InputEvent);
    fn frame(&mut self, state: ApplicationState, delta: f32);
    fn ui(&mut self, state: ApplicationState, context: &Context);
}
