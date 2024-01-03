use crate::ui::state::WidgetState;
use crate::ui::state::WidgetStateTrait;
use capybara::egui::Color32;
use capybara::egui::Image;
use capybara::egui::ImageButton;
use capybara::egui::Label;
use capybara::egui::Response;
use capybara::egui::RichText;
use capybara::egui::Ui;
use capybara::renderer::context::RendererContext;
use capybara::ui::context::UiContext;
use capybara::ui::ImageAtlas;

pub fn button_primary(ui: &mut Ui, context: &UiContext, renderer: &RendererContext, label: &str, state: &mut WidgetState) -> Response {
    button(ui, context, renderer, "button_primary", label, Color32::from_rgb(40, 70, 30), state)
}

pub fn button_secondary(ui: &mut Ui, context: &UiContext, renderer: &RendererContext, label: &str, state: &mut WidgetState) -> Response {
    button(ui, context, renderer, "button_secondary", label, Color32::from_rgb(120, 50, 0), state)
}

pub fn button(
    ui: &mut Ui,
    context: &UiContext,
    renderer: &RendererContext,
    texture: &str,
    label: &str,
    label_color: Color32,
    state: &mut WidgetState,
) -> Response {
    let atlas_handle = context.handles.get("ui").unwrap();
    let atlas_texture = renderer.textures.get_by_name("ui").unwrap();

    let tint = if state.pressed {
        Color32::from_rgba_premultiplied(220, 220, 220, 255)
    } else if state.hovered {
        Color32::from_rgba_premultiplied(230, 230, 230, 255)
    } else {
        Color32::from_rgba_premultiplied(255, 255, 255, 255)
    };

    let mut response = ui.add(ImageButton::new(Image::from_atlas(atlas_handle, atlas_texture, texture).unwrap()).tint(tint).frame(false));
    *state = response.get_state();

    response.rect.set_height(response.rect.height() - 6.0);
    ui.put(response.rect, Label::new(RichText::new(label).size(32.0).color(label_color)));

    response
}
