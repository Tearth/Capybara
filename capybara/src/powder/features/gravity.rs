use crate::powder::chunk::ParticleData;
use crate::powder::chunk::ParticleState;
use crate::powder::simulation::PowderSimulation;
use crate::powder::ParticleDefinition;
use glam::IVec2;
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;

pub fn simulate<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    simulation: &mut PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>,
    definitions: &[ParticleDefinition],
    center_particle: Rc<RefCell<ParticleData>>,
    delta: f32,
) {
    let mut center_particle = center_particle.borrow_mut();
    let center_definition = &definitions[center_particle.r#type];
    let mut center_velocity = center_particle.velocity;

    let top_position = center_particle.position + IVec2::new(0, 1);
    let bottom_position = center_particle.position - IVec2::new(0, 1);

    let top_particle = simulation.get_particle(top_position);
    let top_particle_borrowed = top_particle.as_ref().map(|p| p.borrow());
    let top_type = top_particle_borrowed.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let top_state = top_particle_borrowed.as_ref().map(|p| p.state).unwrap_or(ParticleState::Unknown);

    let bottom_particle = simulation.get_particle(bottom_position);
    let bottom_particle_borrowed = bottom_particle.as_ref().map(|p| p.borrow());
    let bottom_type = bottom_particle_borrowed.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let bottom_state = bottom_particle_borrowed.as_ref().map(|p| p.state).unwrap_or(ParticleState::Unknown);
    let bottom_velocity = bottom_particle_borrowed.as_ref().map(|p| p.velocity).unwrap_or(Vec2::ZERO);
    let mut apply_gravity = true;

    if top_particle.is_some() {
        let top_definition = &definitions[top_type];
        if top_state == ParticleState::Fluid && center_definition.density < top_definition.density {
            center_velocity -= simulation.gravity * delta;
            center_velocity = center_velocity.max(Vec2::ZERO);
            apply_gravity = false;
        }
    } else if bottom_particle.is_some() {
        let bottom_definition = &definitions[bottom_type];
        if bottom_state == ParticleState::Fluid && center_definition.density > bottom_definition.density {
            center_velocity += simulation.gravity * delta;
            center_velocity = center_velocity.min(Vec2::ZERO);
        } else if bottom_state == ParticleState::Powder {
            center_velocity += simulation.gravity * delta;
            center_velocity = center_velocity.max(bottom_velocity + simulation.gravity);
        } else {
            center_velocity = Vec2::ZERO;
        }

        apply_gravity = false;
    }

    if apply_gravity {
        center_velocity += simulation.gravity * delta;
    }

    center_particle.velocity = center_velocity;
}
