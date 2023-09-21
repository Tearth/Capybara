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
use capybara::glam::Vec2;
use capybara::glam::Vec4;
use capybara::instant::Instant;
use capybara::renderer::particles::ParticleEmitter;
use capybara::renderer::particles::ParticleInterpolation;
use capybara::renderer::particles::ParticleParameter;
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
    emitter: ParticleEmitter<5>,
    initialized: bool,
    delta_history: VecDeque<f32>,
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

        if !self.initialized && state.global.assets.load("./data/data0.zip") == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(&state.global.assets, None);
            state.ui.instantiate_assets(&state.global.assets, None);
            state.window.set_swap_interval(0);

            self.emitter.size = Vec2::new(32.0, 8.0);
            self.emitter.period = 0.02;
            self.emitter.amount = 20;
            self.emitter.particle_size = Some(Vec2::new(16.0, 16.0));
            self.emitter.particle_lifetime = 1.0;
            self.emitter.particle_texture_id = Some(state.renderer.textures.get_id("Particle")?);
            self.emitter.interpolation = ParticleInterpolation::Cosine;

            self.emitter.velocity_waypoints.push(ParticleParameter::new(Vec2::new(0.0, 200.0), Vec2::new(100.0, 40.0)));
            self.emitter.velocity_waypoints.push(ParticleParameter::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)));

            self.emitter.scale_waypoints.push(ParticleParameter::new(Vec2::new(1.0, 1.0), Vec2::new(0.5, 0.5)));
            self.emitter.scale_waypoints.push(ParticleParameter::new(Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)));

            self.emitter.color_waypoints.push(ParticleParameter::new(Vec4::new(1.0, 1.0, 0.0, 0.0), Vec4::new(0.0, 0.0, 0.0, 0.0)));
            self.emitter.color_waypoints.push(ParticleParameter::new(Vec4::new(1.0, 1.0, 0.0, 0.2), Vec4::new(0.0, 0.0, 0.0, 0.0)));
            self.emitter.color_waypoints.push(ParticleParameter::new(Vec4::new(1.0, 0.2, 0.2, 0.2), Vec4::new(0.0, 0.0, 0.0, 0.0)));
            self.emitter.color_waypoints.push(ParticleParameter::new(Vec4::new(1.0, 0.2, 0.2, 0.1), Vec4::new(0.0, 0.0, 0.0, 0.0)));
            self.emitter.color_waypoints.push(ParticleParameter::new(Vec4::new(1.0, 0.2, 0.2, 0.0), Vec4::new(0.0, 0.0, 0.0, 0.0)));

            self.initialized = true;
        }

        if self.initialized {
            let cursor_position = Vec2::new(state.window.cursor_position.x as f32, state.window.cursor_position.y as f32);
            self.emitter.position = state.renderer.cameras.get(state.renderer.active_camera_id)?.from_window_to_screen_coordinates(cursor_position);
            self.emitter.update(Instant::now(), delta);
            self.emitter.draw(state.renderer);
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

                    let particles_count = self.emitter.particles.len();
                    let label = format!("N: {}", particles_count);

                    ui.label(RichText::new(label).font(font).heading().color(color));
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
    ApplicationContext::<GlobalData>::new("Particles", WindowStyle::Window { size: Coordinates::new(1280, 720) })?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
