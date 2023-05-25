use crate::ui::state::WidgetState;
use crate::ui::widgets;
use orion_core::anyhow::Result;
use orion_core::app::ApplicationState;
use orion_core::egui::panel::Side;
use orion_core::egui::panel::TopBottomSide;
use orion_core::egui::Align;
use orion_core::egui::CentralPanel;
use orion_core::egui::Color32;
use orion_core::egui::Frame;
use orion_core::egui::FullOutput;
use orion_core::egui::Id;
use orion_core::egui::Layout;
use orion_core::egui::Margin;
use orion_core::egui::RawInput;
use orion_core::egui::RichText;
use orion_core::egui::Rounding;
use orion_core::egui::SidePanel;
use orion_core::egui::Stroke;
use orion_core::egui::TopBottomPanel;
use orion_core::egui::Vec2;
use orion_core::glam::Vec4;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::utils::color::Vec4Color;
use orion_core::window::InputEvent;

#[derive(Default)]
pub struct AboutScene {
    return_button_state: WidgetState,
}

impl Scene for AboutScene {
    fn activation(&mut self, state: ApplicationState) -> Result<()> {
        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, state: ApplicationState, event: InputEvent) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, state: ApplicationState, delta: f32) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let mut command = None;
        let output = state.ui.inner.run(input, |context| {
            let side_panel_width = (context.screen_rect().width() - 420.0) / 2.0;

            TopBottomPanel::new(TopBottomSide::Top, Id::new("top_menu_panel"))
                .exact_height(200.0)
                .frame(Frame::none())
                .show_separator_line(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Template".to_string()).size(160.0).color(Color32::from_rgb(255, 255, 255)));
                    })
                });
            TopBottomPanel::new(TopBottomSide::Bottom, Id::new("bottom_menu_panel"))
                .exact_height(30.0)
                .frame(Frame::none())
                .show_separator_line(false)
                .resizable(false)
                .show(context, |ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("Template v0.1".to_string()).heading().color(Color32::from_rgb(255, 255, 255)));
                    });
                });
            SidePanel::new(Side::Left, Id::new("left_menu_panel"))
                .exact_width(side_panel_width)
                .frame(Frame::none())
                .show_separator_line(false)
                .resizable(false)
                .show(context, |_| {});
            SidePanel::new(Side::Right, Id::new("right_menu_panel"))
                .exact_width(side_panel_width)
                .frame(Frame::none())
                .show_separator_line(false)
                .resizable(false)
                .show(context, |_| {});
            CentralPanel::default().frame(Frame::none()).show(context, |ui| {
                Frame::none()
                    .inner_margin(Margin::symmetric(20.0, 20.0))
                    .stroke(Stroke::new(3.0, Color32::from_rgb(40, 100, 30)))
                    .fill(Color32::from_rgb(180, 220, 160))
                    .rounding(Rounding::same(5.0))
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(
                                    "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras eleifend et mi sit amet convallis. \
                                    Fusce posuere eget ligula non facilisis. Cras vulputate suscipit ipsum faucibus convallis. \
                                    Sed maximus ultricies libero, non varius erat cursus vulputate. Curabitur consequat, dui at semper \
                                    accumsan, mi dui blandit libero, a viverra leo urna nec ligula.",
                                )
                                .color(Color32::from_rgb(40, 70, 30)),
                            );
                            ui.add_space(20.0);

                            if widgets::image_button(ui, state.ui, "button", Vec2::new(190.0, 49.0), "Back", &mut self.return_button_state).clicked() {
                                command = Some(FrameCommand::ChangeScene { name: "MenuScene".to_string() });
                            }
                        });
                    });
            });
        });

        Ok((output, command))
    }
}
