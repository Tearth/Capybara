use super::GlobalData;
use super::*;
use crate::ui::components;
use crate::ui::state::WidgetState;
use capybara::anyhow::Result;
use capybara::app::ApplicationState;
use capybara::egui::panel::TopBottomSide;
use capybara::egui::Align;
use capybara::egui::Align2;
use capybara::egui::Color32;
use capybara::egui::Context;
use capybara::egui::Frame;
use capybara::egui::FullOutput;
use capybara::egui::Grid;
use capybara::egui::Id;
use capybara::egui::Layout;
use capybara::egui::RawInput;
use capybara::egui::RichText;
use capybara::egui::Rounding;
use capybara::egui::Slider;
use capybara::egui::TopBottomPanel;
use capybara::egui::Vec2;
use capybara::egui::Window;
use capybara::glam::Vec4;
use capybara::kira::tween::Tween;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::utils::color::Vec4Color;
use capybara::window::InputEvent;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuSubScene {
    Main,
    Settings,
    About,
}

pub struct MenuScene {
    sub_scene: MenuSubScene,
    settings: Option<SettingsData>,

    play_button_state: WidgetState,
    settings_button_state: WidgetState,
    about_button_state: WidgetState,
    back_button_state: WidgetState,

    #[cfg(not(web))]
    exit_button_state: WidgetState,
}

#[derive(Copy, Clone, Default)]
pub struct SettingsData {
    music_level: f32,
    sound_level: f32,
}

impl MenuScene {
    pub fn subscene_main(&mut self, state: &mut ApplicationState<GlobalData>, context: &Context) -> Option<FrameCommand> {
        let mut command = None;
        let center = context.screen_rect().center();

        Window::new("Main menu")
            .frame(components::frame())
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -50.0))
            .current_pos(center)
            .default_width(200.0)
            .show(context, |ui| {
                ui.vertical_centered(|ui| {
                    if components::button_primary(ui, state.ui, state.renderer, "Play", &mut self.play_button_state).clicked() {
                        command = Some(FrameCommand::ChangeScene { name: "GameScene".to_string() });
                    }

                    ui.add_space(32.0);

                    if components::button_primary(ui, state.ui, state.renderer, "Settings", &mut self.settings_button_state).clicked() {
                        self.sub_scene = MenuSubScene::Settings;
                    }

                    ui.add_space(32.0);

                    if components::button_primary(ui, state.ui, state.renderer, "About", &mut self.about_button_state).clicked() {
                        self.sub_scene = MenuSubScene::About;
                    }

                    #[cfg(not(web))]
                    {
                        ui.add_space(32.0);

                        if components::button_primary(ui, state.ui, state.renderer, "Exit", &mut self.exit_button_state).clicked() {
                            state.window.close();
                        }
                    }
                });
            });

        command
    }

    pub fn subscene_settings(&mut self, state: &mut ApplicationState<GlobalData>, context: &Context) -> Option<FrameCommand> {
        let command = None;
        let center = context.screen_rect().center();

        if self.settings.is_none() {
            self.settings = Some(SettingsData {
                music_level: state.global.settings.get(SETTINGS_MUSIC_LEVEL).unwrap(),
                sound_level: state.global.settings.get(SETTINGS_SOUND_LEVEL).unwrap(),
            });
        }

        Window::new("Settings")
            .frame(components::frame())
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -50.0))
            .current_pos(center)
            .default_width(200.0)
            .show(context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().slider_width = 330.0;
                    ui.visuals_mut().widgets.inactive.rounding = Rounding::same(5.0);
                    ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(50, 50, 50, 255);
                    ui.visuals_mut().selection.bg_fill = Color32::from_rgba_unmultiplied(100, 100, 100, 255);
                    ui.visuals_mut().slider_trailing_fill = true;

                    Grid::new("SettingsGrid").min_row_height(24.0).show(ui, |ui| {
                        let music_track = state.global.music_track.as_ref().unwrap();
                        let sound_track = state.global.sound_track.as_ref().unwrap();

                        ui.label("Music level:");
                        if ui.add(Slider::new(&mut self.settings.as_mut().unwrap().music_level, 0.0..=1.0).show_value(false)).changed() {
                            music_track.set_volume(self.settings.unwrap().music_level as f64, Tween::default()).unwrap();
                        }
                        ui.end_row();

                        ui.label("Sound level:");
                        if ui.add(Slider::new(&mut self.settings.as_mut().unwrap().sound_level, 0.0..=1.0).show_value(false)).changed() {
                            sound_track.set_volume(self.settings.unwrap().sound_level as f64, Tween::default()).unwrap();
                        }
                        ui.end_row();
                    });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if components::button_secondary(ui, state.ui, state.renderer, "Save", &mut self.play_button_state).clicked() {
                            state.global.settings.set(SETTINGS_MUSIC_LEVEL, self.settings.unwrap().music_level, true);
                            state.global.settings.set(SETTINGS_SOUND_LEVEL, self.settings.unwrap().sound_level, true);

                            self.settings = None;
                            self.sub_scene = MenuSubScene::Main;
                        }

                        ui.add_space(32.0);

                        if components::button_primary(ui, state.ui, state.renderer, "Back", &mut self.back_button_state).clicked() {
                            let music_track = state.global.music_track.as_ref().unwrap();
                            let sound_track = state.global.sound_track.as_ref().unwrap();

                            music_track.set_volume(state.global.settings.get::<f64>(SETTINGS_MUSIC_LEVEL).unwrap(), Tween::default()).unwrap();
                            sound_track.set_volume(state.global.settings.get::<f64>(SETTINGS_SOUND_LEVEL).unwrap(), Tween::default()).unwrap();

                            self.settings = None;
                            self.sub_scene = MenuSubScene::Main;
                        }
                    });
                });
            });

        command
    }

    fn subscene_about(&mut self, state: &mut ApplicationState<GlobalData>, context: &Context) -> Option<FrameCommand> {
        let command = None;
        let center = context.screen_rect().center();

        Window::new("About")
            .frame(components::frame())
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -50.0))
            .current_pos(center)
            .default_width(400.0)
            .show(context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(15.0);

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

                    if components::button_primary(ui, state.ui, state.renderer, "Back", &mut self.back_button_state).clicked() {
                        self.sub_scene = MenuSubScene::Main;
                    }
                });
            });

        command
    }
}

impl Scene<GlobalData> for MenuScene {
    fn activation(&mut self, state: ApplicationState<GlobalData>) -> Result<()> {
        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState<GlobalData>, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn fixed(&mut self, _: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, _: ApplicationState<GlobalData>, _: f32, _: f32) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn ui(&mut self, mut state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let mut command = None;
        let output = state.ui.inner.clone().read().unwrap().run(input, |context| {
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

            command = match self.sub_scene {
                MenuSubScene::Main => self.subscene_main(&mut state, context),
                MenuSubScene::Settings => self.subscene_settings(&mut state, context),
                MenuSubScene::About => self.subscene_about(&mut state, context),
            };
        });

        Ok((output, command))
    }
}

impl Default for MenuScene {
    fn default() -> Self {
        Self {
            sub_scene: MenuSubScene::Main,
            settings: None,

            play_button_state: Default::default(),
            settings_button_state: Default::default(),
            about_button_state: Default::default(),
            back_button_state: Default::default(),

            #[cfg(not(web))]
            exit_button_state: Default::default(),
        }
    }
}
