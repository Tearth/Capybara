use capybara::egui::Color32;
use capybara::egui::Frame;
use capybara::egui::Margin;
use capybara::egui::Rounding;
use capybara::egui::Stroke;

pub fn frame() -> Frame {
    Frame::none()
        .inner_margin(Margin::symmetric(20.0, 20.0))
        .stroke(Stroke::new(3.0, Color32::from_rgb(40, 100, 30)))
        .fill(Color32::from_rgb(180, 220, 160))
        .rounding(Rounding::same(5.0))
}
