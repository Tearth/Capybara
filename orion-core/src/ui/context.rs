use crate::renderer::camera::{Camera, CameraOrigin};
use crate::renderer::context::RendererContext;
use crate::window::InputEvent;
use egui::epaint::ahash::HashMap;
use egui::epaint::Primitive;
use egui::FullOutput;
use egui::Pos2;
use egui::RawInput;
use egui::Rect;
use egui::TextureId;
use egui::{Event, ImageData};
use glam::Vec2;
use glow::{Buffer, HasContext, Texture, VertexArray};
use std::rc::Rc;

pub struct UiContext {
    pub inner: egui::Context,
    pub screen_size: Vec2,
    pub collected_events: Vec<Event>,

    pub vao: VertexArray,
    pub vbo: Buffer,
    pub ebo: Buffer,
    pub camera_id: usize,
    pub textures: HashMap<TextureId, Texture>,

    gl: Rc<glow::Context>,
}

impl UiContext {
    pub fn new(renderer: &mut RendererContext) -> Self {
        unsafe {
            let mut context = Self {
                inner: Default::default(),
                screen_size: Default::default(),
                collected_events: Default::default(),
                camera_id: renderer.cameras.store(Camera::new(Vec2::new(0.0, 0.0), renderer.viewport_size, CameraOrigin::LeftTop)),
                vao: renderer.gl.create_vertex_array().unwrap(),
                vbo: renderer.gl.create_buffer().unwrap(),
                ebo: renderer.gl.create_buffer().unwrap(),
                textures: Default::default(),
                gl: renderer.gl.clone(),
            };
            context.init();

            context
        }
    }

    fn init(&mut self) {
        unsafe {
            let f32_size = core::mem::size_of::<f32>() as i32;

            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            self.gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 9 * f32_size, 0);
            self.gl.vertex_attrib_pointer_f32(1, 4, glow::FLOAT, false, 9 * f32_size, 3 * f32_size);
            self.gl.vertex_attrib_pointer_f32(2, 2, glow::FLOAT, false, 9 * f32_size, 7 * f32_size);

            self.gl.enable_vertex_attrib_array(0);
            self.gl.enable_vertex_attrib_array(1);
            self.gl.enable_vertex_attrib_array(2);
        }
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
        let mut input = RawInput::default();
        input.screen_rect = Some(Rect::from_two_pos(Pos2::new(0.0, 0.0), Pos2::new(self.screen_size.x, self.screen_size.y)));
        input.events = self.collected_events.clone();
        input.max_texture_side = unsafe { Some(self.gl.get_parameter_i32(glow::MAX_TEXTURE_SIZE) as usize) };

        self.collected_events.clear();

        input
    }

    pub fn draw(&mut self, renderer: &mut RendererContext, output: FullOutput) {
        let clipped_primitives = self.inner.tessellate(output.shapes);
        renderer.activate_camera(self.camera_id);

        for (id, delta) in output.textures_delta.set {
            if let ImageData::Font(font) = delta.image {
                unsafe {
                    let texture_id = self.gl.create_texture().unwrap();
                    let data: Vec<u8> = font.srgba_pixels(None).flat_map(|a| a.to_array()).collect();

                    self.gl.bind_texture(glow::TEXTURE_2D, Some(texture_id));
                    self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::MIRRORED_REPEAT as i32);
                    self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::MIRRORED_REPEAT as i32);
                    self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST_MIPMAP_NEAREST as i32);
                    self.gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
                    self.gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA8 as i32,
                        font.size[0] as i32,
                        font.size[1] as i32,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        Some(&data),
                    );
                    self.gl.generate_mipmap(glow::TEXTURE_2D);
                    self.textures.insert(id, texture_id);
                }
            }
        }

        for shape in clipped_primitives {
            if let Primitive::Mesh(mesh) = shape.primitive {
                let mut data = Vec::new();
                for vertice in mesh.vertices {
                    data.push(vertice.pos.x);
                    data.push(vertice.pos.y);
                    data.push(-1.0);
                    data.push(vertice.color.r() as f32 / 255.0);
                    data.push(vertice.color.g() as f32 / 255.0);
                    data.push(vertice.color.b() as f32 / 255.0);
                    data.push(vertice.color.a() as f32 / 255.0);
                    data.push(vertice.uv.x);
                    data.push(vertice.uv.y);
                }

                unsafe {
                    let f32_size = core::mem::size_of::<f32>() as i32;
                    let vertices_u8 = core::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * f32_size as usize);
                    let indices_u8 = core::slice::from_raw_parts(mesh.indices.as_ptr() as *const u8, mesh.indices.len() * f32_size as usize);

                    self.gl.bind_vertex_array(Some(self.vao));
                    self.gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
                    self.gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

                    self.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
                    self.gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STATIC_DRAW);

                    self.gl.bind_texture(glow::TEXTURE_2D, Some(*self.textures.get(&mesh.texture_id).unwrap()));
                    self.gl.draw_elements(glow::TRIANGLES, mesh.indices.len() as i32, glow::UNSIGNED_INT, 0);
                }
            }
        }
    }
}
