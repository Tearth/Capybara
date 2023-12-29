use crate::utils::debug::DebugCollectorData;
use capybara::egui::Color32;
use capybara::egui::Context;
use capybara::egui::Frame;
use capybara::egui::Margin;
use capybara::egui::Rounding;
use capybara::egui::Stroke;
use capybara::egui::Window;

pub fn debug_window(context: &Context, data: &DebugCollectorData) {
    Window::new("Debug window")
        .frame(debug_frame())
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .default_width(600.0)
        .show(context, |ui| {
            ui.style_mut().visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(255, 255, 255));

            ui.label(&data.hardware_info);
            ui.add_space(15.0);
            ui.columns(2, |ui| {
                ui[0].vertical(|ui| {
                    ui.label(format!("FPS current: {:.1}", data.fps_current));
                    ui.label(format!("FPS average: {}", data.fps_average));
                    ui.label(format!("Delta current: {:.1} ms", data.delta_current * 1000.0));
                    ui.label(format!("Delta average: {:.1} ms", data.delta_average * 1000.0));
                    ui.label(format!("Delta deviation: {:.1} ms", data.delta_deviation * 1000.0));
                });
                ui[1].vertical(|ui| {
                    ui.label("Test");
                    ui.label("Test");
                    ui.label("Test");
                });
            });
        });
}

fn debug_frame() -> Frame {
    Frame::none()
        .inner_margin(Margin::symmetric(10.0, 10.0))
        .stroke(Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 0, 0, 220)))
        .fill(Color32::from_rgba_unmultiplied(40, 40, 40, 220))
        .rounding(Rounding::same(5.0))
}
