use crate::assets::RawTexture;
use crate::renderer::camera::{Camera, CameraOrigin};
use crate::renderer::context::RendererContext;
use crate::renderer::sprite::{Shape, ShapeData, Sprite};
use crate::renderer::texture::Texture;
use crate::window::InputEvent;
use anyhow::Result;
use egui::epaint::ahash::HashMap;
use egui::epaint::Primitive;
use egui::FullOutput;
use egui::Pos2;
use egui::RawInput;
use egui::Rect;
use egui::TextureId;
use egui::{Event, ImageData};
use glam::Vec2;
use glow::HasContext;
use std::rc::Rc;

pub struct UiContext {
    pub inner: egui::Context,
    pub screen_size: Vec2,
    pub collected_events: Vec<Event>,

    pub camera_id: usize,
    pub textures: HashMap<TextureId, usize>,

    gl: Rc<glow::Context>,
}

impl UiContext {
    pub fn new(renderer: &mut RendererContext) -> Result<Self> {
        unsafe {
            let mut context = Self {
                inner: Default::default(),
                screen_size: Default::default(),
                collected_events: Default::default(),

                camera_id: renderer.cameras.store(Camera::new(Default::default(), renderer.viewport_size, CameraOrigin::LeftTop)),
                textures: Default::default(),

                gl: renderer.gl.clone(),
            };
            context.init(renderer)?;

            Ok(context)
        }
    }

    fn init(&mut self, renderer: &mut RendererContext) -> Result<()> {
        unsafe { Ok(()) }
    }

    pub fn collect_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::WindowSizeChange { size } => {
                self.screen_size = Vec2::new(size.x as f32, size.y as f32);
            }
            InputEvent::MouseMove { position, modifiers } => self.collected_events.push(Event::PointerMoved(Pos2::new(position.x as f32, position.y as f32))),
            _ => {}
        }
    }

    pub fn get_input(&mut self) -> RawInput {
        unsafe {
            let mut input = RawInput::default();
            input.screen_rect = Some(Rect::from_two_pos(Pos2::new(0.0, 0.0), Pos2::new(self.screen_size.x, self.screen_size.y)));
            input.events = self.collected_events.clone();
            input.max_texture_side = Some(self.gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as usize);

            self.collected_events.clear();

            input
        }
    }

    pub fn draw(&mut self, renderer: &mut RendererContext, output: FullOutput) -> Result<()> {
        renderer.activate_camera(self.camera_id)?;

        for (id, delta) in output.textures_delta.set {
            if let ImageData::Font(font) = delta.image {
                let data: Vec<u8> = font.srgba_pixels(None).flat_map(|a| a.to_array()).collect();

                if let Some(position) = delta.pos {
                    let texture_id = self.textures.get(&id).unwrap();
                    let texture = renderer.textures.get(*texture_id)?;
                    texture.update(Vec2::new(position[0] as f32, position[1] as f32), Vec2::new(font.size[0] as f32, font.size[1] as f32), data);
                } else {
                    let raw = RawTexture::new("".to_string(), Vec2::new(font.size[0] as f32, font.size[1] as f32), data);
                    let texture_id = renderer.textures.store(Texture::new(self.gl.clone(), &raw));
                    self.textures.insert(id, texture_id);
                }
            }
        }

        for shape in self.inner.tessellate(output.shapes) {
            if let Primitive::Mesh(mesh) = shape.primitive {
                let mut vertices = Vec::new();
                for vertice in mesh.vertices {
                    vertices.push(vertice.pos.x);
                    vertices.push(vertice.pos.y);
                    vertices.push(vertice.color.r() as f32 / 255.0);
                    vertices.push(vertice.color.g() as f32 / 255.0);
                    vertices.push(vertice.color.b() as f32 / 255.0);
                    vertices.push(vertice.color.a() as f32 / 255.0);
                    vertices.push(vertice.uv.x);
                    vertices.push(vertice.uv.y);
                }

                let mut sprite = Sprite::new();
                sprite.shape = Shape::Custom(ShapeData::new(vertices, mesh.indices));
                sprite.texture_id = *self.textures.get(&mesh.texture_id).unwrap();

                renderer.draw(&sprite)?;
                renderer.flush()?;
            }
        }

        for id in output.textures_delta.free {
            let texture_id = self.textures.get(&id).unwrap();
            renderer.textures.remove(*texture_id)?;
        }

        Ok(())
    }
}
