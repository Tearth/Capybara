use super::GlobalData;
use crate::ui::components;
use crate::ui::state::WidgetState;
use crate::utils::console::Console;
use crate::utils::debug::DebugCollector;
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
use capybara::utils::profiler::Profiler;
use capybara::window::InputEvent;
use capybara::window::Key;

#[derive(Default)]
pub struct GameScene {
    play_button_state: WidgetState,
    exit_button_state: WidgetState,
    exit_menu_visible: bool,

    debug_enabled: bool,
    debug_console: Console,
    debug_profiler: Profiler,
    debug_collector: DebugCollector,
}

impl Scene<GlobalData> for GameScene {
    fn activation(&mut self, state: ApplicationState<GlobalData>) -> Result<()> {
        self.exit_menu_visible = false;

        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));
        Ok(())
    }

    fn deactivation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        self.debug_profiler.resume("input");

        if let InputEvent::KeyPress { key, repeat: _, modifiers } = event {
            if key == Key::Escape {
                self.exit_menu_visible = !self.exit_menu_visible;
            } else if key == Key::KeyD && modifiers.shift {
                self.debug_enabled = !self.debug_enabled;
                self.debug_profiler.enabled = !self.debug_profiler.enabled;
                self.debug_collector.enabled = !self.debug_collector.enabled;
            }
        }

        self.debug_profiler.pause("input");
        Ok(())
    }

    fn fixed(&mut self, _state: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        self.debug_profiler.start("fixed");
        self.debug_profiler.stop("fixed");

        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, _accumulator: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.debug_profiler.start("frame");

        if self.debug_enabled {
            self.debug_collector.collect(&state, delta);
            self.process_console();
        }
        self.debug_profiler.stop("frame");

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        self.debug_profiler.start("ui");
        self.debug_profiler.stop("input");

        let mut command = None;
        let output = state.ui.inner.read().unwrap().run(input, |context| {
            let center = context.screen_rect().center();

            if self.exit_menu_visible {
                Window::new("Back to the menu? The game will be lost")
                    .frame(components::frame())
                    .movable(false)
                    .resizable(false)
                    .collapsible(false)
                    .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, 0.0))
                    .current_pos(center)
                    .show(context, |ui| {
                        ui.add_space(15.0);
                        ui.horizontal(|ui| {
                            if components::button_primary(ui, state.ui, state.renderer, "Yes", &mut self.play_button_state).clicked() {
                                command = Some(FrameCommand::ChangeScene { name: "MenuScene".to_string() });
                            }

                            ui.add_space(32.0);

                            if components::button_secondary(ui, state.ui, state.renderer, "No", &mut self.exit_button_state).clicked() {
                                self.exit_menu_visible = false;
                            }
                        });
                    });
            }

            if self.debug_enabled {
                components::debug_window(context, &mut self.debug_console, &self.debug_profiler, &mut self.debug_collector);
            }
        });

        self.debug_profiler.stop("ui");
        Ok((output, command))
    }
}

impl GameScene {
    pub fn process_console(&mut self) {
        while let Some(command) = self.debug_console.poll_command() {
            match command.to_lowercase().as_str() {
                "test" => self.debug_console.apply_output("Test"),
                _ => self.debug_console.apply_output("Invalid command"),
            }
        }
    }
}
