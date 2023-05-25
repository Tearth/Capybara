use orion_core::egui::Response;

#[derive(Default)]
pub struct WidgetState {
    pub pressed: bool,
    pub hovered: bool,
}

pub trait WidgetStateTrait {
    fn get_state(&self) -> WidgetState;
}

impl WidgetStateTrait for Response {
    fn get_state(&self) -> WidgetState {
        WidgetState { pressed: self.is_pointer_button_down_on(), hovered: self.hovered() }
    }
}
