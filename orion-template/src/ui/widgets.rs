use super::state::WidgetState;
use super::state::WidgetStateTrait;
use orion_core::egui::Color32;
use orion_core::egui::ImageButton;
use orion_core::egui::Label;
use orion_core::egui::Response;
use orion_core::egui::RichText;
use orion_core::egui::Ui;
use orion_core::egui::Vec2;
use orion_core::ui::context::UiContext;

pub fn image_button(ui: &mut Ui, context: &UiContext, texture: &str, size: Vec2, label: &str, state: &mut WidgetState) -> Response {
    let tint = if state.pressed {
        Color32::from_rgba_premultiplied(220, 220, 220, 255)
    } else if state.hovered {
        Color32::from_rgba_premultiplied(230, 230, 230, 255)
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, 255)
    };

    let mut response = ui.add(ImageButton::new(context.handles.get(texture).unwrap(), size).tint(tint).frame(false));
    *state = response.get_state();

    response.rect.set_height(response.rect.height() - 6.0);
    ui.put(response.rect, Label::new(RichText::new(label).color(Color32::from_rgb(40, 70, 30))));

    response
}
