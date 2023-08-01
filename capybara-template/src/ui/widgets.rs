use super::state::WidgetState;
use super::state::WidgetStateTrait;
use capybara_core::egui::Color32;
use capybara_core::egui::Frame;
use capybara_core::egui::ImageButton;
use capybara_core::egui::Label;
use capybara_core::egui::Margin;
use capybara_core::egui::Response;
use capybara_core::egui::RichText;
use capybara_core::egui::Rounding;
use capybara_core::egui::Stroke;
use capybara_core::egui::Ui;
use capybara_core::egui::Vec2;
use capybara_core::ui::context::UiContext;

pub fn button_green(ui: &mut Ui, context: &UiContext, label: &str, state: &mut WidgetState) -> Response {
    image_button(ui, context, "button_green", Vec2::new(190.0, 49.0), label, Color32::from_rgb(40, 70, 30), state)
}

pub fn button_orange(ui: &mut Ui, context: &UiContext, label: &str, state: &mut WidgetState) -> Response {
    image_button(ui, context, "button_orange", Vec2::new(190.0, 49.0), label, Color32::from_rgb(120, 50, 0), state)
}

pub fn image_button(
    ui: &mut Ui,
    context: &UiContext,
    texture: &str,
    size: Vec2,
    label: &str,
    label_color: Color32,
    state: &mut WidgetState,
) -> Response {
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
    ui.put(response.rect, Label::new(RichText::new(label).size(26.0).color(label_color)));

    response
}

pub fn frame() -> Frame {
    Frame::none()
        .inner_margin(Margin::symmetric(20.0, 20.0))
        .stroke(Stroke::new(3.0, Color32::from_rgb(40, 100, 30)))
        .fill(Color32::from_rgb(180, 220, 160))
        .rounding(Rounding::same(5.0))
}
