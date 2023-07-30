use orion_core::anyhow::Result;
use orion_core::app::ApplicationContext;
use orion_core::app::ApplicationState;
use orion_core::assets::AssetsLoadingStatus;
use orion_core::egui::panel::Side;
use orion_core::egui::Color32;
use orion_core::egui::FontFamily;
use orion_core::egui::FontId;
use orion_core::egui::FullOutput;
use orion_core::egui::Id;
use orion_core::egui::RawInput;
use orion_core::egui::RichText;
use orion_core::egui::SidePanel;
use orion_core::fast_gpu;
use orion_core::fastrand;
use orion_core::glam::Vec2;
use orion_core::renderer::sprite::Sprite;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::Key;
use orion_core::window::WindowStyle;
use std::collections::VecDeque;

fast_gpu!();

#[derive(Default)]
struct GlobalData {}

#[derive(Default)]
struct MainScene {
    objects: Vec<Object>,
    initialized: bool,
    delta_history: VecDeque<f32>,
}

struct Object {
    sprite: Sprite,
    direction: Vec2,
}

impl Scene<GlobalData> for MainScene {
    fn activation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        if let InputEvent::KeyPress { key: Key::Escape, repeat: _, modifiers: _ } = event {
            state.window.close();
        }

        Ok(())
    }

    fn fixed(&mut self, _: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, _: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.delta_history.push_back(delta);

        if self.delta_history.len() > 100 {
            self.delta_history.pop_front();
        }

        if !self.initialized && state.assets.load("./data/data0.zip")? == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(state.assets, None)?;
            state.ui.instantiate_assets(state.assets, None)?;
            state.window.set_swap_interval(0);

            for _ in 0..200000 {
                let position = Vec2::new(
                    fastrand::u32(0..state.renderer.viewport_size.x as u32) as f32,
                    fastrand::u32(0..state.renderer.viewport_size.y as u32) as f32,
                );

                self.objects.push(Object {
                    sprite: Sprite { position, texture_id: Some(state.renderer.textures.get_by_name("Takodachi")?.id), ..Default::default() },
                    direction: Vec2::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0),
                });
            }

            self.initialized = true;
        }

        for object in &mut self.objects {
            object.sprite.position += object.direction * 100.0 * delta;

            if object.sprite.position.x < 0.0 {
                object.direction = Vec2::new(object.direction.x.abs(), object.direction.y);
            } else if object.sprite.position.x > state.renderer.viewport_size.x {
                object.direction = Vec2::new(-object.direction.x.abs(), object.direction.y);
            } else if object.sprite.position.y < 0.0 {
                object.direction = Vec2::new(object.direction.x, object.direction.y.abs());
            } else if object.sprite.position.y > state.renderer.viewport_size.y {
                object.direction = Vec2::new(object.direction.x, -object.direction.y.abs());
            }

            state.renderer.draw_sprite(&object.sprite)?;
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.run(input, |context| {
            SidePanel::new(Side::Left, Id::new("side")).resizable(false).show(context, |ui| {
                if self.initialized {
                    let font = FontId { size: 24.0, family: FontFamily::Name("Kenney Pixel".into()) };
                    let color = Color32::from_rgb(255, 255, 255);
                    let label = format!("FPS: {}", state.renderer.fps);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
                    let label = format!("Delta: {:.2}", delta_average * 1000.0);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));
                    ui.label(RichText::new(format!("N: {}", self.objects.len())).font(font).heading().color(color));
                }
            });
        });

        Ok((output, None))
    }
}

fn main() {
    ApplicationContext::<GlobalData>::new("Benchmark", WindowStyle::Window { size: Coordinates::new(1280, 720) })
        .unwrap()
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene")
        .unwrap();
}
