use crate::assets::RawTexture;
use crate::error_return;
use crate::glam::IVec2;
use crate::glam::Vec2;
use crate::glam::Vec4;
use crate::renderer::context::RendererContext;
use crate::renderer::shape::Shape;
use crate::renderer::shape::ShapeVertex;
use crate::renderer::sprite::TextureId;
use crate::renderer::texture::Texture;
use crate::utils::color::Vec4Utils;

pub struct Canvas<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32> {
    pub chunk_position: IVec2,
    pub shape: Option<Shape>,
    pub texture_id: usize,
    pub texture_data: Vec<u8>,
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32> Canvas<CHUNK_SIZE, PARTICLE_SIZE> {
    pub fn initialize(&mut self, renderer: &mut RendererContext, chunk_position: IVec2) {
        let name = format!("canvas_{}_{}", chunk_position.x, chunk_position.y);
        let size = Vec2::new(CHUNK_SIZE as f32, CHUNK_SIZE as f32);

        let raw = RawTexture::new(&name, "", size, &self.texture_data);
        let texture = Texture::new(renderer, &raw).expect("Failed to create texture");
        self.texture_id = renderer.textures.store(texture);

        let white = Vec4::ONE.to_rgb_packed();
        let width_real = (CHUNK_SIZE * PARTICLE_SIZE) as f32;

        let canvas_left_bottom = Vec2::new(0.0, 0.0);
        let canvas_right_bottom = Vec2::new(width_real, 0.0);
        let canvas_right_top = Vec2::new(width_real, width_real);
        let canvas_left_top = Vec2::new(0.0, width_real);

        let mut shape = Shape::new();
        shape.apply_model = true;
        shape.texture_id = TextureId::Some(self.texture_id);
        shape.vertices.push(ShapeVertex::new(canvas_left_bottom, white, Vec2::new(0.0, 0.0)));
        shape.vertices.push(ShapeVertex::new(canvas_right_bottom, white, Vec2::new(1.0, 0.0)));
        shape.vertices.push(ShapeVertex::new(canvas_right_top, white, Vec2::new(1.0, 1.0)));
        shape.vertices.push(ShapeVertex::new(canvas_left_top, white, Vec2::new(0.0, 1.0)));
        shape.indices = vec![0, 1, 2, 0, 2, 3];

        self.chunk_position = chunk_position;
        self.shape = Some(shape);
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        let shape = match &mut self.shape {
            Some(shape) => shape,
            None => error_return!("Canvas shape is not initialized"),
        };
        shape.position = self.chunk_position.as_vec2() * (CHUNK_SIZE * PARTICLE_SIZE) as f32;

        renderer.draw_shape(shape);
    }

    pub fn set_particle(&mut self, position: IVec2, color: Vec4) {
        let index = self.position_to_texture_index(position & (CHUNK_SIZE - 1));
        self.texture_data[index * 4 + 0] = (color.x * 255.0) as u8;
        self.texture_data[index * 4 + 1] = (color.y * 255.0) as u8;
        self.texture_data[index * 4 + 2] = (color.z * 255.0) as u8;
        self.texture_data[index * 4 + 3] = (color.w * 255.0) as u8;
    }

    pub fn update_texture(&mut self, renderer: &RendererContext) {
        let texture = match renderer.textures.get(self.texture_id) {
            Ok(texture) => texture,
            Err(err) => error_return!("Failed to update canvas texture ({})", err),
        };

        texture.update(Vec2::ZERO, texture.size, &self.texture_data);
    }

    pub fn position_to_texture_index(&self, position: IVec2) -> usize {
        (position.x + position.y * CHUNK_SIZE) as usize
    }
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32> Default for Canvas<CHUNK_SIZE, PARTICLE_SIZE> {
    fn default() -> Self {
        Self {
            chunk_position: Default::default(),
            shape: Default::default(),
            texture_id: Default::default(),
            texture_data: [0, 0, 0, 255].repeat((CHUNK_SIZE * CHUNK_SIZE) as usize),
        }
    }
}
