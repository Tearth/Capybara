use crate::ui::state::WidgetState;
use crate::ui::widgets;
use orion_core::anyhow::Result;
use orion_core::app::ApplicationState;
use orion_core::egui::panel::TopBottomSide;
use orion_core::egui::Align;
use orion_core::egui::Align2;
use orion_core::egui::Color32;
use orion_core::egui::Context;
use orion_core::egui::Frame;
use orion_core::egui::FullOutput;
use orion_core::egui::Id;
use orion_core::egui::Layout;
use orion_core::egui::RawInput;
use orion_core::egui::RichText;
use orion_core::egui::TopBottomPanel;
use orion_core::egui::Vec2;
use orion_core::egui::Window;
use orion_core::glam::Vec4;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::utils::color::Vec4Color;
use orion_core::window::InputEvent;

pub enum MenuSubScene {
    Main,
    About,
}

pub struct MenuScene {
    sub_scene: MenuSubScene,

    play_button_state: WidgetState,
    about_button_state: WidgetState,
    exit_button_state: WidgetState,
    back_button_state: WidgetState,
}

impl MenuScene {
    pub fn subscene_main(&mut self, state: &ApplicationState, context: &Context) -> Option<FrameCommand> {
        let mut command = None;
        let center = context.screen_rect().center();

        Window::new("Main menu")
            .frame(widgets::frame())
            .movable(false)
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .anchor(Align2::CENTER_CENTER, Vec2::new(0.0, -50.0))
            .current_pos(center)
            .default_width(200.0)
            .show(context, |ui| {
                ui.vertical_centered(|ui| {
                    if widgets::button_green(ui, state.ui, "Play", &mut self.play_button_state).clicked() {
                        command = Some(FrameCommand::ChangeScene { name: "GameScene".to_string() });
                    }

                    ui.add_space(32.0);

                    if widgets::button_green(ui, state.ui, "About", &mut self.about_button_state).clicked() {
                        self.sub_scene = MenuSubScene::About;
                    }

                    ui.add_space(32.0);

                    if widgets::button_green(ui, state.ui, "Exit", &mut self.exit_button_state).clicked() {
                        state.window.close();
                    }
                });
            });

        command
    }

    fn subscene_about(&mut self, state: &ApplicationState, context: &Context) -> Option<FrameCommand> {
        let command = None;
        let center = context.screen_rect().center();

        Window::new("About")
            .frame(widgets::frame())
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

                    if widgets::button_green(ui, state.ui, "Back", &mut self.back_button_state).clicked() {
                        self.sub_scene = MenuSubScene::Main;
                    }
                });
            });

        command
    }
}

impl Scene for MenuScene {
    fn activation(&mut self, state: ApplicationState) -> Result<()> {
        state.renderer.set_clear_color(Vec4::new_rgb(40, 80, 30, 255));
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, _: ApplicationState, _: InputEvent) -> Result<()> {
        Ok(())
    }

    fn frame(&mut self, _: ApplicationState, _: f32) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let mut command = None;
        let output = state.ui.inner.run(input, |context| {
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
                MenuSubScene::Main => self.subscene_main(&state, context),
                MenuSubScene::About => self.subscene_about(&state, context),
            };
        });

        Ok((output, command))
    }
}

impl Default for MenuScene {
    fn default() -> Self {
        Self {
            sub_scene: MenuSubScene::Main,
            play_button_state: Default::default(),
            about_button_state: Default::default(),
            exit_button_state: Default::default(),
            back_button_state: Default::default(),
        }
    }
}
