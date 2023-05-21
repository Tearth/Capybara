use orion_core::anyhow::Result;
use orion_core::app::ApplicationState;
use orion_core::egui;
use orion_core::egui::panel::Side;
use orion_core::egui::panel::TopBottomSide;
use orion_core::egui::Align;
use orion_core::egui::CentralPanel;
use orion_core::egui::Color32;
use orion_core::egui::Direction;
use orion_core::egui::Frame;
use orion_core::egui::FullOutput;
use orion_core::egui::Id;
use orion_core::egui::ImageData;
use orion_core::egui::Layout;
use orion_core::egui::RawInput;
use orion_core::egui::RichText;
use orion_core::egui::Sense;
use orion_core::egui::SidePanel;
use orion_core::egui::TopBottomPanel;
use orion_core::egui::Vec2;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::window::InputEvent;

#[derive(Default)]
pub struct MenuScene {
    texture: Option<egui::TextureHandle>,
}

impl Scene for MenuScene {
    fn activation(&mut self, _: ApplicationState) -> Result<()> {
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

    fn ui(&mut self, state: ApplicationState, input: RawInput) -> Result<FullOutput> {
        Ok(state.ui.inner.run(input, |context| {
            let frame = Frame::default();

            TopBottomPanel::new(TopBottomSide::Top, Id::new("top")).exact_height(150.0).frame(frame).show(context, |ui| {});
            TopBottomPanel::new(TopBottomSide::Bottom, Id::new("bottom")).exact_height(150.0).frame(frame).show(context, |ui| {});
            SidePanel::new(Side::Left, Id::new("left")).frame(frame).show(context, |ui| {});
            SidePanel::new(Side::Right, Id::new("right")).frame(frame).show(context, |ui| {});

            CentralPanel::default().show(context, |ui| {
                ui.vertical_centered(|ui| {
                    ui.button("TEST 1");
                    ui.add_space(20.0);
                    ui.button("TEST 2");
                    ui.add_space(20.0);
                    ui.button("TEST 3");
                });
            });
        }))
    }
}
