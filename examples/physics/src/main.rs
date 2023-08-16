use capybara::anyhow::Result;
use capybara::app::ApplicationContext;
use capybara::app::ApplicationState;
use capybara::assets::loader::AssetsLoader;
use capybara::assets::AssetsLoadingStatus;
use capybara::egui::panel::Side;
use capybara::egui::Color32;
use capybara::egui::FontFamily;
use capybara::egui::FontId;
use capybara::egui::FullOutput;
use capybara::egui::Id;
use capybara::egui::RawInput;
use capybara::egui::RichText;
use capybara::egui::SidePanel;
use capybara::fast_gpu;
use capybara::fastrand;
use capybara::glam::Vec2;
use capybara::glam::Vec4;
use capybara::rapier2d::prelude::*;
use capybara::renderer::sprite::Sprite;
use capybara::scene::FrameCommand;
use capybara::scene::Scene;
use capybara::window::Coordinates;
use capybara::window::InputEvent;
use capybara::window::Key;
use capybara::window::WindowStyle;
use std::collections::VecDeque;

fast_gpu!();

#[derive(Default)]
struct GlobalData {
    assets: AssetsLoader,
}

#[derive(Default)]
struct MainScene {
    objects: Vec<Object>,
    initialized: bool,
    delta_history: VecDeque<f32>,

    terrain: Sprite,
    terrain_collider: Option<ColliderHandle>,

    wheel_left: Sprite,
    wheel_right: Sprite,
    wheel_left_rigidbody: RigidBodyHandle,
    wheel_right_rigidbody: RigidBodyHandle,

    car: Sprite,
    car_rigidbody: RigidBodyHandle,
}

struct Object {
    sprite: Sprite,
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
        if let InputEvent::KeyPress { key: Key::Escape, repeat: _, modifiers: _ } = event {
            state.window.close();
        }

        Ok(())
    }

    fn fixed(&mut self, state: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        let collisions = state.physics.events.collisions.read().unwrap();
        let contacts = state.physics.events.contacts.read().unwrap();

        println!("Collisions: {}, contacts: {}", collisions.len(), contacts.len());

        if state.window.keyboard_state[Key::KeyA as usize] {
            state.physics.rigidbodies.get_mut(self.wheel_left_rigidbody).unwrap().apply_torque_impulse(0.02, true);
        }

        if state.window.keyboard_state[Key::KeyD as usize] {
            state.physics.rigidbodies.get_mut(self.wheel_left_rigidbody).unwrap().apply_torque_impulse(-0.02, true);
        }

        if state.window.keyboard_state[Key::Space as usize] {
            state.physics.rigidbodies.get_mut(self.wheel_left_rigidbody).unwrap().set_angvel(0.0, true);
        }

        Ok(None)
    }

    fn frame(&mut self, state: ApplicationState<GlobalData>, accumulator: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.delta_history.push_back(delta);

        if self.delta_history.len() > 100 {
            self.delta_history.pop_front();
        }

        const PIXELS_PER_METER: f32 = 50.0;

        if !self.initialized && state.global.assets.load("./data/data0.zip")? == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(&state.global.assets, None)?;
            state.ui.instantiate_assets(&state.global.assets, None)?;
            state.window.set_swap_interval(0);

            self.terrain = Sprite { size: Some(Vec2::new(state.renderer.viewport_size.x, 50.0)), ..Default::default() };
            self.terrain_collider = Some(state.physics.colliders.insert(ColliderBuilder::cuboid(100.0, 0.1).build()));

            self.wheel_left = Sprite { texture_id: Some(state.renderer.textures.get_id("Wheel")?), ..Default::default() };
            self.wheel_right = Sprite { texture_id: Some(state.renderer.textures.get_id("Wheel")?), ..Default::default() };
            self.car = Sprite { size: Some(Vec2::new(100.0, 50.0)), color: Vec4::new(0.8, 0.8, 0.8, 1.0), ..Default::default() };

            let collider = ColliderBuilder::ball(0.3).restitution(0.7).build();
            let rigidbody = RigidBodyBuilder::dynamic().translation(vector![300.0, 300.0] / PIXELS_PER_METER).build();
            self.wheel_left_rigidbody = state.physics.rigidbodies.insert(rigidbody);
            state.physics.colliders.insert_with_parent(collider, self.wheel_left_rigidbody, &mut state.physics.rigidbodies);

            let collider = ColliderBuilder::ball(0.3).restitution(0.7).build();
            let rigidbody = RigidBodyBuilder::dynamic().translation(vector![350.0, 300.0] / PIXELS_PER_METER).build();
            self.wheel_right_rigidbody = state.physics.rigidbodies.insert(rigidbody);
            state.physics.colliders.insert_with_parent(collider, self.wheel_right_rigidbody, &mut state.physics.rigidbodies);

            let collider = ColliderBuilder::cuboid(1.0, 0.5).restitution(0.7).build();
            let rigidbody = RigidBodyBuilder::dynamic().translation(vector![325.0, 320.0] / PIXELS_PER_METER).build();
            self.car_rigidbody = state.physics.rigidbodies.insert(rigidbody);
            state.physics.colliders.insert_with_parent(collider, self.car_rigidbody, &mut state.physics.rigidbodies);

            let joint = RevoluteJointBuilder::new().local_anchor1(point![-0.6, -0.5]).local_anchor2(point![0.0, 0.0]).contacts_enabled(false);
            state.physics.impulse_joints.insert(self.car_rigidbody, self.wheel_left_rigidbody, joint, true);

            let joint = RevoluteJointBuilder::new().local_anchor1(point![0.6, -0.5]).local_anchor2(point![0.0, 0.0]).contacts_enabled(false);
            state.physics.impulse_joints.insert(self.car_rigidbody, self.wheel_right_rigidbody, joint, true);

            for _ in 0..20 {
                let position = Vec2::new(
                    fastrand::u32(0..state.renderer.viewport_size.x as u32) as f32,
                    fastrand::u32(0..state.renderer.viewport_size.y as u32) as f32,
                );
                let sprite = Sprite { position, texture_id: Some(state.renderer.textures.get_id("Takodachi")?), ..Default::default() };
                let collider_flags = ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS;
                let collider = ColliderBuilder::ball(0.3).restitution(0.7).active_events(collider_flags).build();
                let rigidbody = RigidBodyBuilder::dynamic().translation(vector![position.x, position.y] / PIXELS_PER_METER).build();
                let rigidbody_handle = state.physics.rigidbodies.insert(rigidbody);
                state.physics.colliders.insert_with_parent(collider, rigidbody_handle, &mut state.physics.rigidbodies);

                self.objects.push(Object { sprite, rigidbody: rigidbody_handle });
            }

            self.initialized = true;
        }

        if self.initialized {
            let alpha = accumulator / state.physics.integration_parameters.dt;

            for i in 0..self.objects.len() {
                let object = &mut self.objects[i];
                if let Some(interpolation_data) = state.physics.interpolation_data.get(&object.rigidbody) {
                    self.objects[i].sprite.position = interpolation_data.get_position_interpolated(alpha) * PIXELS_PER_METER;
                    self.objects[i].sprite.rotation = interpolation_data.get_rotation_interpolated(alpha);
                    state.renderer.draw_sprite(&self.objects[i].sprite)?;
                }
            }

            if let Some(interpolation_data) = state.physics.interpolation_data.get(&self.wheel_left_rigidbody) {
                self.wheel_left.position = interpolation_data.get_position_interpolated(alpha) * PIXELS_PER_METER;
                self.wheel_left.rotation = interpolation_data.get_rotation_interpolated(alpha);
                state.renderer.draw_sprite(&self.wheel_left)?;
            }

            if let Some(interpolation_data) = state.physics.interpolation_data.get(&self.wheel_right_rigidbody) {
                self.wheel_right.position = interpolation_data.get_position_interpolated(alpha) * PIXELS_PER_METER;
                self.wheel_right.rotation = interpolation_data.get_rotation_interpolated(alpha);
                state.renderer.draw_sprite(&self.wheel_right)?;
            }

            if let Some(interpolation_data) = state.physics.interpolation_data.get(&self.car_rigidbody) {
                self.car.position = interpolation_data.get_position_interpolated(alpha) * PIXELS_PER_METER;
                self.car.rotation = interpolation_data.get_rotation_interpolated(alpha);
                state.renderer.draw_sprite(&self.car)?;
            }
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().unwrap().run(input, |context| {
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
    main_internal().unwrap();
}

fn main_internal() -> Result<()> {
    ApplicationContext::<GlobalData>::new("Physics", WindowStyle::Window { size: Coordinates::new(1280, 720) })?
        .with_scene("MainScene", Box::<MainScene>::default())?
        .run("MainScene")
        .unwrap();

    Ok(())
}
