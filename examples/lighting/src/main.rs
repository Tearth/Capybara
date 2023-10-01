use capybara::anyhow::Result;
use capybara::app::ApplicationContext;
use capybara::app::ApplicationState;
use capybara::assets::loader::AssetsLoader;
use capybara::assets::AssetsLoadingStatus;
use capybara::assets::RawTexture;
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
use capybara::renderer::lighting::LightEmitter;
use capybara::renderer::shader::Shader;
use capybara::renderer::sprite::Sprite;
use capybara::renderer::sprite::TextureId;
use capybara::renderer::texture::Texture;
use capybara::renderer::Edge;
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
    emitter: LightEmitter,

    main_texture_id: usize,
    light_texture_id: usize,
    mult_shader_id: usize,
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

    fn input(&mut self, mut state: ApplicationState<GlobalData>, event: InputEvent) -> Result<()> {
        if let InputEvent::KeyPress { key: Key::Escape, repeat: _, modifiers: _ } = event {
            state.window.close();
        } else if let InputEvent::WindowSizeChange { size } = event {
            self.update_shaders_resolution(&mut state, size)?;
        }

        Ok(())
    }

    fn fixed(&mut self, _: ApplicationState<GlobalData>) -> Result<Option<FrameCommand>> {
        Ok(None)
    }

    fn frame(&mut self, mut state: ApplicationState<GlobalData>, _: f32, delta: f32) -> Result<Option<FrameCommand>> {
        self.delta_history.push_back(delta);

        if self.delta_history.len() > 100 {
            self.delta_history.pop_front();
        }

        if !self.initialized && state.global.assets.load("./data/data0.zip") == AssetsLoadingStatus::Finished {
            state.renderer.instantiate_assets(&state.global.assets, None);
            state.ui.instantiate_assets(&state.global.assets, None);
            state.window.set_swap_interval(0);

            for _ in 0..200 {
                let position = Vec2::new(
                    fastrand::u32(0..state.renderer.viewport_size.x as u32) as f32,
                    fastrand::u32(0..state.renderer.viewport_size.y as u32) as f32,
                );

                self.objects.push(Object {
                    sprite: Sprite { position, texture_id: TextureId::Some(state.renderer.textures.get_id("Takodachi")?), ..Default::default() },
                    direction: Vec2::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0),
                });
            }

            let target_texture = Texture::new(&state.renderer, &RawTexture::new("target_texture", "", Vec2::new(400.0, 400.0), &Vec::new()))?;
            self.light_texture_id = state.renderer.textures.store(target_texture);

            let main_texture = Texture::new(&state.renderer, &RawTexture::new("main_texture", "", Vec2::new(400.0, 400.0), &Vec::new()))?;
            self.main_texture_id = state.renderer.textures.store(main_texture);

            let mult_shader = Shader::new(&state.renderer, "mult", include_str!("./shaders/mult.vert"), include_str!("./shaders/mult.frag"))?;
            mult_shader.activate();
            mult_shader.set_uniform("mainSampler", &0.0);
            mult_shader.set_uniform("lightSampler", &1.0);
            self.mult_shader_id = state.renderer.shaders.store(mult_shader);

            let resolution = state.renderer.viewport_size.into();
            self.update_shaders_resolution(&mut state, resolution)?;

            self.initialized = true;
        }

        if self.initialized {
            let mut edges = Vec::new();
            let texture_size = state.renderer.textures.get_by_name("Takodachi")?.size;

            state.renderer.set_target_texture(Some(self.main_texture_id), true);
            state.renderer.draw_sprite(&Sprite {
                anchor: Vec2::new(0.0, 0.0),
                size: Some(state.renderer.viewport_size),
                color: Vec4::new(0.5, 0.5, 0.5, 1.0),
                ..Default::default()
            });
            for object in &mut self.objects {
                if object.sprite.position.x < 0.0 {
                    object.direction = Vec2::new(object.direction.x.abs(), object.direction.y);
                } else if object.sprite.position.x > state.renderer.viewport_size.x {
                    object.direction = Vec2::new(-object.direction.x.abs(), object.direction.y);
                } else if object.sprite.position.y < 0.0 {
                    object.direction = Vec2::new(object.direction.x, object.direction.y.abs());
                } else if object.sprite.position.y > state.renderer.viewport_size.y {
                    object.direction = Vec2::new(object.direction.x, -object.direction.y.abs());
                }

                state.renderer.draw_sprite(&object.sprite);
                edges.append(&mut object.sprite.get_edges(texture_size));
            }
            state.renderer.set_target_texture(None, true);

            let min = Vec2::new(0.0, 0.0);
            let max = state.renderer.viewport_size;

            edges.append(&mut vec![
                Edge::new(min, Vec2::new(max.x, min.y)),
                Edge::new(Vec2::new(max.x, min.y), Vec2::new(max.x, max.y)),
                Edge::new(Vec2::new(max.x, max.y), Vec2::new(min.x, max.y)),
                Edge::new(Vec2::new(min.x, max.y), Vec2::new(min.x, min.y)),
            ]);

            self.emitter.position = state.renderer.cameras.get(0)?.from_window_to_screen_coordinates(state.window.cursor_position.into());
            self.emitter.edges = edges;
            self.emitter.debug = true;

            let response = self.emitter.generate();
            state.renderer.set_target_texture(Some(self.light_texture_id), true);
            state.renderer.clear();
            state.renderer.draw_shape(&response.shape);
            state.renderer.set_target_texture(None, true);

            state.renderer.set_sprite_shader(Some(self.mult_shader_id));
            state.renderer.textures.get(self.main_texture_id)?.activate(0);
            state.renderer.textures.get(self.light_texture_id)?.activate(1);

            state.renderer.draw_sprite(&Sprite {
                texture_id: TextureId::None,
                anchor: Vec2::new(0.0, 0.0),
                size: Some(state.renderer.viewport_size),
                ..Default::default()
            });
            state.renderer.set_sprite_shader(None);
            //self.emitter.draw_debug(state.renderer, &response);
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().unwrap().run(input, |context| {
            SidePanel::new(Side::Left, Id::new("side")).exact_width(120.0).resizable(false).show(context, |ui| {
                if self.initialized {
                    let font = FontId { size: 24.0, family: FontFamily::Name("Kenney Pixel".into()) };
                    let color = Color32::from_rgb(255, 255, 255);
                    let label = format!("FPS: {}", state.renderer.fps);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
                    let label = format!("Delta: {:.2}", delta_average * 1000.0);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));
                    ui.label(RichText::new(format!("N: {}", self.objects.len())).font(font.clone()).heading().color(color));
                }
            });
        });

        Ok((output, None))
    }
}

impl MainScene {
    fn update_shaders_resolution(&mut self, state: &mut ApplicationState<GlobalData>, size: Coordinates) -> Result<()> {
        for shader in state.renderer.shaders.iter_mut() {
            if shader.uniforms.contains_key("resolution") {
                shader.activate();
                shader.set_uniform("resolution", [size.x as f32, size.y as f32].as_ptr());
            }
        }

        if state.renderer.selected_shader_id != usize::MAX {
            state.renderer.shaders.get_mut(state.renderer.selected_shader_id)?.activate();
        }

        Ok(())
    }
}

fn main() {
    main_internal().unwrap();
}

fn main_internal() -> Result<()> {
    ApplicationContext::<GlobalData>::new("Lighting", WindowStyle::Window { size: Coordinates::new(1280, 720) })?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
