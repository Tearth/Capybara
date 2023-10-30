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
use capybara::egui::Slider;
use capybara::egui::TextStyle;
use capybara::fast_gpu;
use capybara::fastrand;
use capybara::glam::Vec2;
use capybara::renderer::shader::Shader;
use capybara::renderer::sprite::Sprite;
use capybara::renderer::sprite::TextureId;
use capybara::renderer::sprite::TextureType;
use capybara::renderer::texture::Texture;
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
    objects_count: u32,
    initialized: bool,
    delta_history: VecDeque<f32>,
    target_texture_id: usize,
    selected_shader: SelectedShader,

    blur_shader_id: usize,
    grayscale_shader_id: usize,
}

struct Object {
    sprite: Sprite,
    direction: Vec2,
}

#[derive(Default, PartialEq)]
enum SelectedShader {
    #[default]
    None,
    Blur,
    Grayscale,
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

            self.regenerate_objects(&mut state, 100000)?;

            let target_texture = Texture::new(&state.renderer, &RawTexture::new("target_texture", "", Vec2::new(400.0, 400.0), &Vec::new()))?;
            self.target_texture_id = state.renderer.textures.store(target_texture);

            let blur_shader = Shader::new(&state.renderer, "blur", include_str!("./shaders/blur.vert"), include_str!("./shaders/blur.frag"))?;
            self.blur_shader_id = state.renderer.shaders.store(blur_shader);

            let grayscale_shader =
                Shader::new(&state.renderer, "grayscale", include_str!("./shaders/grayscale.vert"), include_str!("./shaders/grayscale.frag"))?;
            self.grayscale_shader_id = state.renderer.shaders.store(grayscale_shader);

            let resolution = state.renderer.viewport_size.into();
            self.update_shaders_resolution(&mut state, resolution)?;

            self.initialized = true;
        }

        if self.initialized && self.selected_shader != SelectedShader::None {
            state.renderer.set_target_texture(Some(self.target_texture_id));
            state.renderer.clear();
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

            state.renderer.draw_sprite(&object.sprite);
        }

        if self.initialized && self.selected_shader != SelectedShader::None {
            let shader_id = match self.selected_shader {
                SelectedShader::Blur => self.blur_shader_id,
                SelectedShader::Grayscale => self.grayscale_shader_id,
                _ => panic!("Invalid shader"),
            };

            state.renderer.set_target_texture(None);
            state.renderer.set_sprite_shader(Some(shader_id));

            state.renderer.draw_sprite(&Sprite {
                texture_id: TextureId::Some(self.target_texture_id),
                texture_type: TextureType::Simple,
                anchor: Vec2::new(0.0, 0.0),
                ..Default::default()
            });

            state.renderer.set_sprite_shader(None);
        }

        Ok(None)
    }

    fn ui(&mut self, state: ApplicationState<GlobalData>, input: RawInput) -> Result<(FullOutput, Option<FrameCommand>)> {
        let output = state.ui.inner.read().unwrap().run(input, |context| {
            SidePanel::new(Side::Left, Id::new("side")).exact_width(160.0).resizable(false).show(context, |ui| {
                if self.initialized {
                    let font = FontId { size: 24.0, family: FontFamily::Name("Kenney Pixel".into()) };
                    let color = Color32::from_rgb(255, 255, 255);
                    let label = format!("FPS: {}", state.renderer.fps);

                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    let delta_average = self.delta_history.iter().sum::<f32>() / self.delta_history.len() as f32;
                    let label = format!("Delta: {:.2}", delta_average * 1000.0);
                    ui.label(RichText::new(label).font(font.clone()).heading().color(color));

                    ui.style_mut().drag_value_text_style = TextStyle::Monospace;
                    ui.style_mut().text_styles.get_mut(&TextStyle::Monospace).unwrap().size = 20.0;

                    ui.add_space(10.0);
                    ui.label(RichText::new("Objects count:").font(font.clone()).heading().color(color));
                    if ui.add(Slider::new(&mut self.objects_count, 0..=1000000).text_color(color).logarithmic(true)).changed() {
                        self.regenerate_objects(&state, self.objects_count).unwrap();
                    }

                    ui.add_space(10.0);
                    ui.label(RichText::new("Shaders:").font(font.clone()).heading().color(color));
                    ui.radio_value(&mut self.selected_shader, SelectedShader::None, RichText::new("None").font(font.clone()).heading().color(color));
                    ui.radio_value(&mut self.selected_shader, SelectedShader::Blur, RichText::new("Blur").font(font.clone()).heading().color(color));
                    ui.radio_value(
                        &mut self.selected_shader,
                        SelectedShader::Grayscale,
                        RichText::new("Grayscale").font(font.clone()).heading().color(color),
                    );
                }
            });
        });

        Ok((output, None))
    }
}

impl MainScene {
    fn regenerate_objects(&mut self, state: &ApplicationState<GlobalData>, n: u32) -> Result<()> {
        self.objects.clear();
        self.objects_count = n;

        for _ in 0..n {
            let position = Vec2::new(
                fastrand::u32(0..state.renderer.viewport_size.x as u32) as f32,
                fastrand::u32(0..state.renderer.viewport_size.y as u32) as f32,
            );

            self.objects.push(Object {
                sprite: Sprite { position, texture_id: TextureId::Some(state.renderer.textures.get_id("Takodachi")?), ..Default::default() },
                direction: Vec2::new(fastrand::f32() * 2.0 - 1.0, fastrand::f32() * 2.0 - 1.0),
            });
        }

        Ok(())
    }

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
    ApplicationContext::<GlobalData>::new("Benchmark", WindowStyle::Window { size: Coordinates::new(1280, 720) }, Some(8))?
        .with_scene("MainScene", Box::<MainScene>::default())
        .run("MainScene");

    Ok(())
}
