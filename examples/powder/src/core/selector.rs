use super::*;
use capybara::fastrand;
use capybara::glam::IVec2;
use capybara::glam::Vec4;
use capybara::powder::chunk::ParticleData;
use capybara::powder::chunk::ParticleState;
use capybara::powder::simulation::PowderSimulation;
use capybara::powder::ParticleDefinition;
use capybara::renderer::context::RendererContext;
use capybara::renderer::shape::Shape;

#[derive(Default)]
pub struct Selector {
    pub position: IVec2,
    pub size: IVec2,
    pub shape: Option<Shape>,
    pub particle_type: usize,
    pub particle_definition: Option<ParticleDefinition>,
}

impl Selector {
    pub fn draw(&mut self, renderer: &mut RendererContext) {
        if let Some(shape) = &self.shape {
            renderer.draw_shape(shape);
        }
    }

    pub fn update(&mut self) {
        let canvas_left_bottom = self.position - (self.size - 1) / 2;
        let canvas_right_top = self.position + (self.size - 1) / 2;
        let selector_left_bottom = canvas_left_bottom * PARTICLE_SIZE;
        let selector_right_top = canvas_right_top * PARTICLE_SIZE + PARTICLE_SIZE;

        let color = if let Some(particle) = &self.particle_definition { particle.color } else { Vec4::new(0.5, 0.5, 0.5, 1.0) };
        self.shape = Some(Shape::new_frame(selector_left_bottom.as_vec2(), selector_right_top.as_vec2(), PARTICLE_SIZE as f32, color));
    }

    pub fn reset(&mut self) {
        self.position = IVec2::ZERO;
        self.size = IVec2::ONE;
        self.shape = None;
    }

    pub fn fill_selection(&self, simulation: &mut PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>) {
        let mut last_position = None;
        let mut particles = Vec::new();

        if let Some(definition) = &self.particle_definition {
            while let Some(position) = self.get_next_selected_particle(last_position) {
                particles.push(ParticleData {
                    r#type: self.particle_type,
                    state: definition.state,
                    position,
                    color: definition.color,
                    hpressure: if definition.state == ParticleState::Fluid { 1.0 } else { 0.0 },
                    ..Default::default()
                });

                last_position = Some(position);
            }

            while !particles.is_empty() {
                let i = fastrand::usize(0..particles.len());
                let particle = particles[i];

                if !simulation.particle_exists(particle.position) {
                    simulation.add_particle(particle.position, particle);
                }

                particles.remove(i);
            }
        }
    }

    pub fn clear_selection(&mut self, simulation: &mut PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>) {
        let mut last_position = None;

        while let Some(position) = self.get_next_selected_particle(last_position) {
            if simulation.particle_exists(position) {
                simulation.remove_particle(position);
            }

            last_position = Some(position);
        }
    }

    pub fn get_next_selected_particle(&self, position: Option<IVec2>) -> Option<IVec2> {
        if let Some(position) = position {
            let canvas_left_bottom = self.position - (self.size - 1) / 2;
            let canvas_right_top = self.position + (self.size - 1) / 2;
            let next_position = position + IVec2::new(1, 0);

            if next_position.x <= canvas_right_top.x {
                Some(next_position)
            } else {
                let next_position = IVec2::new(i32::max(0, canvas_left_bottom.x), position.y + 1);
                if next_position.y <= canvas_right_top.y {
                    Some(next_position)
                } else {
                    None
                }
            }
        } else {
            let result = self.position - (self.size - 1) / 2;
            Some(IVec2::new(result.x, result.y))
        }
    }

    pub fn set_cursor_position(&mut self, cursor_position: IVec2) {
        self.position = cursor_position / PARTICLE_SIZE;
        self.update();
    }

    pub fn increase_size(&mut self) {
        self.size = (self.size + 2).clamp(IVec2::ONE, IVec2::MAX);
        self.update();
    }

    pub fn decrease_size(&mut self) {
        self.size = (self.size - 2).clamp(IVec2::ONE, IVec2::MAX);
        self.update();
    }
}
