use super::GlobalData;
use crate::ui::state::WidgetState;
use crate::ui::widgets;
use capybara::anyhow::Result;
use capybara::app::ApplicationState;
use capybara::egui::Align2;
use capybara::egui::FullOutput;
use capybara::egui::RawInput;
use capybara::egui::Vec2;
use capybara::egui::Window;
use capybara::glam::Vec4;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::utils::color::Vec4Color;
use capybara::window::InputEvent;
use capybara::window::Key;

#[derive(Default)]
pub struct GameScene {
    play_button_state: WidgetState,
    exit_button_state: WidgetState,
    exit_menu_visible: bool,
}

impl Scene<GlobalData> for GameScene {
    fn activation(&mut self, state: ApplicationState<GlobalData>) -> Result<()> {
        self.exit_menu_visible = false;

        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        if let InputEvent::KeyPress { key, repeat: _, modifiers: _ } = event {
            if key == Key::Escape {
                self.exit_menu_visible = !self.exit_menu_visible;
            }
        }
        Ok(())
    }

    fn fixed(&mut self, _: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, _: ApplicationState<GlobalData>, _: f32, _: f32) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
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

                            if widgets::button_green(ui, state.ui, "No", &mut self.exit_button_state).clicked() {
                                self.exit_menu_visible = false;
                            }
                        });
                    });
            }
        });

        Ok((output, command))
    }
}
