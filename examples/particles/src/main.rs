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
use orion_core::glam::Vec4;
use orion_core::instant::Instant;
use orion_core::renderer::particles::ParticleEmitter;
use orion_core::renderer::particles::ParticleInterpolation;
use orion_core::renderer::particles::ParticleParameter;
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

        if !self.initialized && state.assets.load("./data/data0.zip")? == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(state.assets, None)?;
            state.ui.instantiate_assets(state.assets, None)?;
            state.window.set_swap_interval(0);

            self.emitter.size = Vec2::new(32.0, 8.0);
            self.emitter.period = 0.02;
            self.emitter.bursts = 0;
            self.emitter.amount = 20;
            self.emitter.particle_size = Some(Vec2::new(16.0, 16.0));
            self.emitter.particle_lifetime = 1.0;
            self.emitter.particle_texture_id = Some(state.renderer.textures.get_by_name("Particle")?.id);
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
            self.emitter.position =
                Vec2::new(state.window.cursor_position.x as f32, state.window.size.y as f32 - state.window.cursor_position.y as f32);
            self.emitter.update(Instant::now(), delta);
            self.emitter.draw(state.renderer);
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
    ApplicationContext::<GlobalData>::new("Particles", WindowStyle::Window { size: Coordinates::new(800, 600) })
        .unwrap()
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene")
        .unwrap();
}
