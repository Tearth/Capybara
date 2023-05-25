use crate::ui::state::WidgetState;
use crate::ui::widgets;
use orion_core::anyhow::Result;
use orion_core::app::ApplicationState;
use orion_core::egui::Align2;
use orion_core::egui::FullOutput;
use orion_core::egui::RawInput;
use orion_core::egui::Vec2;
use orion_core::egui::Window;
use orion_core::glam::Vec4;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::utils::color::Vec4Color;
use orion_core::window::InputEvent;
use orion_core::window::Key;

#[derive(Default)]
pub struct GameScene {
    play_button_state: WidgetState,
    about_button_state: WidgetState,
    exit_menu_visible: bool,
}

impl Scene for GameScene {
    fn activation(&mut self, state: ApplicationState) -> Result<()> {
        self.exit_menu_visible = false;

        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState, event: InputEvent) -> Result<()> {
        if let InputEvent::KeyPress { key, repeat: _, modifiers: _ } = event {
            if key == Key::Escape {
                self.exit_menu_visible = !self.exit_menu_visible;
            }
        }
        Ok(())
    }

    fn frame(&mut self, _: ApplicationState, _: f32) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let mut command = None;
        let output = state.ui.inner.run(input, |context| {
            let center = context.screen_rect().center();

            if self.exit_menu_visible {
                Window::new("Back to the menu? The game will be lost")
                    .frame(widgets::frame())
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                    .current_pos(center)
                    .show(context, |ui| {
                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            if widgets::button_orange(ui, state.ui, "Yes", &mut self.play_button_state).clicked() {
                                command = Some(FrameCommand::ChangeScene { name: "MenuScene".to_string() });
                            }

                            ui.add_space(32.0);

                            if widgets::button_green(ui, state.ui, "No", &mut self.about_button_state).clicked() {
                                self.exit_menu_visible = false;
                            }
                        });
                    });
            }
        });

        Ok((output, command))
    }
}
