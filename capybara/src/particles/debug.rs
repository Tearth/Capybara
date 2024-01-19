use super::emitter::ParticleEmitter;
use crate::renderer::context::RendererContext;
use crate::renderer::shape::Shape;
use glam::Vec4;

#[derive(Debug)]
pub struct ParticlesDebugSettings {
    pub frame_color: Vec4,
    pub frame_thickness: f32,
}

impl Default for ParticlesDebugSettings {
    fn default() -> Self {
        Self { frame_color: Vec4::new(1.0, 1.0, 1.0, 1.0), frame_thickness: 1.0 }
    }
}

impl<const WAYPOINTS: usize> ParticleEmitter<WAYPOINTS> {
    pub fn draw_debug(&self, renderer: &mut RendererContext) {
        for particle in self.particles.iter() {
            let mut shape = Shape::new_frame(-self.particle_size / 2.0, self.particle_size / 2.0, self.debug.frame_thickness, self.debug.frame_color);
            shape.position = particle.postion;
            shape.rotation = particle.rotation;

            renderer.draw_shape(&shape);
        }
    }
}
