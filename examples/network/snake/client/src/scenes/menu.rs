use super::GlobalData;
use super::*;
use crate::network::lobby::LobbyNetworkContext;
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
use capybara::instant::Instant;
use capybara::kira::tween::Tween;
use capybara::log::error;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::utils::color::Vec4Utils;
use capybara::window::InputEvent;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MenuSubScene {
    Main,
    Settings,
    About,
}

pub struct MenuScene {
    sub_scene: MenuSubScene,
    settings: SettingsData,
    network: LobbyNetworkContext,
    initialized: bool,

    play_button_state: WidgetState,
    settings_button_state: WidgetState,
    about_button_state: WidgetState,
    back_button_state: WidgetState,

    #[cfg(not(web))]
    exit_button_state: WidgetState,
}

#[derive(Copy, Clone, Default)]
pub struct SettingsData {
    master_volume: f32,
    music_volume: f32,
    effects_volume: f32,
}

impl Scene<GlobalData> for MenuScene {
    fn activation(&mut self, state: ApplicationState<GlobalData>) -> Result<()> {
        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));

        if !self.initialized {
            let master_volume = state.global.settings.get::<f32>(SETTINGS_MASTER_VOLUME);
            let music_volume = state.global.settings.get::<f32>(SETTINGS_MUSIC_VOLUME);
            let effects_volume = state.global.settings.get::<f32>(SETTINGS_EFFECTS_VOLUME);

            if let Err(ref err) = master_volume {
                error!("Failed to read master volume ({})", err);
            }
            if let Err(ref err) = music_volume {
                error!("Failed to read music volume ({})", err);
            }
            if let Err(ref err) = effects_volume {
                error!("Failed to read effects volume ({})", err);
            }

            self.settings = SettingsData {
                master_volume: master_volume.unwrap_or(1.0),
                music_volume: music_volume.unwrap_or(1.0),
                effects_volume: effects_volume.unwrap_or(1.0),
            };
        }

        Ok(())
    }

    fn deactivation(&mut self, _state: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _state: ApplicationState<GlobalData>, _event: InputEvent) -> Result<()> {
        Ok(())
    }

    fn fixed(&mut self, _state: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, _state: ApplicationState<GlobalData>, _accumulator: f32, _delta: f32) -> Result<Option<FrameCommand>> {
        let now = Instant::now();
        self.network.process(now);

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

    fn reset(&self) -> Box<dyn Scene<GlobalData>> {
        Box::<Self>::default()
    }
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
            .default_width(250.0)
            .show(context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(format!("Welcome {}!", self.network.player_name)).heading());

                    ui.add_space(32.0);

                    if components::button_primary(ui, state.ui, state.renderer, "Play", &mut self.play_button_state).clicked() {
                        if !self.network.servers.is_empty() {
                            state.global.player_name = self.network.player_name.clone();
                            state.global.server_name = self.network.servers[0].name.clone();
                            state.global.server_flag = self.network.servers[0].flag.clone();
                            state.global.server_address = self.network.servers[0].address.clone();

                            command = Some(FrameCommand::ChangeScene { name: "GameScene".to_string() });
                        }
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

        Window::new("Settings")
            .frame(components::frame())
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -50.0))
            .current_pos(center)
            .default_width(200.0)
            .show(context, |ui| {
                ui.add_space(15.0);
                ui.vertical_centered(|ui| {
                    ui.spacing_mut().slider_width = 305.0;
                    ui.visuals_mut().widgets.inactive.rounding = Rounding::same(5.0);
                    ui.visuals_mut().widgets.inactive.bg_fill = Color32::from_rgba_unmultiplied(50, 50, 50, 255);
                    ui.visuals_mut().selection.bg_fill = Color32::from_rgba_unmultiplied(100, 100, 100, 255);
                    ui.visuals_mut().slider_trailing_fill = true;

                    Grid::new("settings_grid").min_row_height(24.0).show(ui, |ui| {
                        let music_track = state.global.music_track.as_ref().unwrap();
                        let effects_track = state.global.effects_track.as_ref().unwrap();

                        let master_volume = &mut self.settings.master_volume;
                        let music_volume = &mut self.settings.music_volume;
                        let effects_volume = &mut self.settings.effects_volume;

                        ui.label("Master Volume:");
                        if ui.add(Slider::new(master_volume, 0.0..=1.0).show_value(false)).changed() {
                            if let Err(err) = music_track.set_volume((*master_volume * *music_volume) as f64, Tween::default()) {
                                error!("Failed to set music volume ({})", err);
                            }

                            if let Err(err) = effects_track.set_volume((*master_volume * *effects_volume) as f64, Tween::default()) {
                                error!("Failed to set effects volume ({})", err);
                            }
                        }
                        ui.end_row();

                        ui.label("Music Volume:");
                        if ui.add(Slider::new(music_volume, 0.0..=1.0).show_value(false)).changed() {
                            if let Err(err) = music_track.set_volume((*master_volume * *music_volume) as f64, Tween::default()) {
                                error!("Failed to set music volume ({})", err);
                            }
                        }
                        ui.end_row();

                        ui.label("Effects Volume:");
                        if ui.add(Slider::new(effects_volume, 0.0..=1.0).show_value(false)).changed() {
                            if let Err(err) = effects_track.set_volume((*master_volume * *effects_volume) as f64, Tween::default()) {
                                error!("Failed to set effects volume ({})", err);
                            }
                        }
                        ui.end_row();
                    });

                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if components::button_primary(ui, state.ui, state.renderer, "Save", &mut self.play_button_state).clicked() {
                            state.global.settings.set(SETTINGS_MASTER_VOLUME, self.settings.master_volume, true);
                            state.global.settings.set(SETTINGS_MUSIC_VOLUME, self.settings.music_volume, true);
                            state.global.settings.set(SETTINGS_EFFECTS_VOLUME, self.settings.effects_volume, true);

                            self.sub_scene = MenuSubScene::Main;
                        }

                        ui.add_space(32.0);

                        if components::button_secondary(ui, state.ui, state.renderer, "Back", &mut self.back_button_state).clicked() {
                            let music_track = state.global.music_track.as_ref().unwrap();
                            let effect_track = state.global.effects_track.as_ref().unwrap();

                            let master_volume = state.global.settings.get::<f64>(SETTINGS_MASTER_VOLUME);
                            let music_volume = state.global.settings.get::<f64>(SETTINGS_MUSIC_VOLUME);
                            let effects_volume = state.global.settings.get::<f64>(SETTINGS_EFFECTS_VOLUME);

                            if let Err(ref err) = master_volume {
                                error!("Failed to read master volume ({})", err);
                            }
                            if let Err(ref err) = music_volume {
                                error!("Failed to read music volume ({})", err);
                            }
                            if let Err(ref err) = effects_volume {
                                error!("Failed to read effects volume ({})", err);
                            }

                            let master_volume = master_volume.unwrap_or(1.0);
                            let music_volume = music_volume.unwrap_or(1.0);
                            let effects_volume = effects_volume.unwrap_or(1.0);

                            if let Err(err) = music_track.set_volume(music_volume * master_volume, Tween::default()) {
                                error!("Failed to set music volume ({})", err);
                            }

                            if let Err(err) = effect_track.set_volume(effects_volume * master_volume, Tween::default()) {
                                error!("Failed to set effects volume ({})", err);
                            }

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
                ui.add_space(15.0);
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

                    if components::button_primary(ui, state.ui, state.renderer, "Back", &mut self.back_button_state).clicked() {
                        self.sub_scene = MenuSubScene::Main;
                    }
                });
            });

        command
    }
}

impl Default for MenuScene {
    fn default() -> Self {
        Self {
            sub_scene: MenuSubScene::Main,
            settings: SettingsData::default(),
            network: LobbyNetworkContext::default(),
            initialized: false,

            play_button_state: WidgetState::default(),
            settings_button_state: WidgetState::default(),
            about_button_state: WidgetState::default(),
            back_button_state: WidgetState::default(),

            #[cfg(not(web))]
            exit_button_state: WidgetState::default(),
        }
    }
}
