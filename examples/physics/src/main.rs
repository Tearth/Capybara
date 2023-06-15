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
use orion_core::glam::Vec2;
use orion_core::rapier2d::prelude::*;
use orion_core::renderer::sprite::Sprite;
use orion_core::scene::FrameCommand;
use orion_core::scene::Scene;
use orion_core::window::Coordinates;
use orion_core::window::InputEvent;
use orion_core::window::Key;
use orion_core::window::WindowStyle;
use std::collections::VecDeque;

fast_gpu!();

const COUNT: usize = 1000;
const DELTA_HISTORY_COUNT: usize = 100;
const PIXELS_PER_METER: f32 = 50.0;

#[derive(Default)]
struct GlobalData {}

#[derive(Default)]
struct MainScene {
    objects: Vec<Object>,
    initialized: bool,
    delta_history: VecDeque<f32>,

    terrain: Sprite,
    terrain_collider: Option<ColliderHandle>,
}

struct Object {
    sprite: Sprite,
    collider: ColliderHandle,
    rigidbody: RigidBodyHandle,
}

impl Scene<GlobalData> for MainScene {
    fn activation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn deactivation(&mut self, _: ApplicationState<GlobalData>) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        if let InputEvent::KeyPress { key, repeat: _, modifiers: _ } = event {
            match key {
                Key::Escape => state.window.close(),
                Key::Space => state.window.set_cursor_visibility(!state.window.cursor_visible),
                _ => {}
            }
        }

        Ok(())
    }

    fn fixed(&mut self, _: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, accumulator: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.delta_history.push_back(delta);

        if self.delta_history.len() > DELTA_HISTORY_COUNT {
            self.delta_history.pop_front();
        }

        if !self.initialized && state.assets.load("./data/data0.zip")? == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(state.assets, None)?;
            state.ui.instantiate_assets(state.assets, None)?;
            state.window.set_swap_interval(0);

            self.terrain = Sprite { size: Some(Vec2::new(state.renderer.viewport_size.x, 50.0)), ..Default::default() };
            self.terrain_collider = Some(state.physics.colliders.insert(ColliderBuilder::cuboid(100.0, 0.1).build()));

            for _ in 0..COUNT {
                let position = Vec2::new(
                    fastrand::u32(0..state.renderer.viewport_size.x as u32) as f32,
                    fastrand::u32(0..state.renderer.viewport_size.y as u32) as f32,
                );
                let sprite = Sprite { position, texture_id: Some(state.renderer.textures.get_by_name("tako")?.id), ..Default::default() };
                let collider = ColliderBuilder::ball(0.3).restitution(0.7).build();
                let rigidbody = RigidBodyBuilder::dynamic().translation(vector![position.x, position.y] / PIXELS_PER_METER).build();
                let rigidbody_handle = state.physics.rigidbodies.insert(rigidbody);
                let collider_handle = state.physics.colliders.insert_with_parent(collider, rigidbody_handle, &mut state.physics.rigidbodies);

                self.objects.push(Object { sprite, collider: collider_handle, rigidbody: rigidbody_handle });
            }

            self.initialized = true;
        }

        if self.initialized {
            let alpha = accumulator / state.physics.integration_parameters.dt;

            for i in 0..COUNT {
                let object = &mut self.objects[i];
                if let Some(interpolation_data) = state.physics.interpolation_data.get(&object.rigidbody) {
                    self.objects[i].sprite.position = interpolation_data.get_position_interpolated(alpha) * PIXELS_PER_METER;
                    self.objects[i].sprite.rotation = interpolation_data.get_rotation_interpolated(alpha);
                    state.renderer.draw_sprite(&self.objects[i].sprite)?;
                }
            }
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.run(input, |context| {
            SidePanel::new(Side::Left, Id::new("side")).show(context, |ui| {
                if self.initialized {
                    let font = FontId { size: 24.0, family: FontFamily::Name("Kenney Pixel".into()) };
                    let color = Color32::from_rgb(255, 255, 255);
                    let label = format!("FPS: {}", state.renderer.fps);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
                    let label = format!("Delta: {:.2}", delta_average * 1000.0);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));
                    ui.label(RichText::new(format!("N: {}", COUNT)).font(font).heading().color(color));
                }
            });
        });

        Ok((output, None))
    }
}

fn main() {
    ApplicationContext::<GlobalData>::new("Physics", WindowStyle::Window { size: Coordinates::new(800, 600) })
        .unwrap()
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene")
        .unwrap();
}
