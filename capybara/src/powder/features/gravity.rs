use crate::powder::chunk::ParticleData;
use crate::powder::chunk::ParticleState;
use crate::powder::local::LocalChunksGuards;
use crate::powder::ParticleDefinition;
use glam::IVec2;
use glam::Vec2;

pub fn simulate(
    local: &mut LocalChunksGuards,
    definitions: &[ParticleDefinition],
    particle_id: usize,
    state: ParticleState,
    gravity: Vec2,
    delta: f32,
) {
    let center_particle = local.chunks[0].get_storage(state).get_unchecked(particle_id);
    let center_definition = &definitions[center_particle.r#type];
    let center_position = center_particle.position;
    let mut center_velocity = center_particle.velocity;

    let top_position = center_particle.position + IVec2::new(0, 1);
    let bottom_position = center_particle.position - IVec2::new(0, 1);

    let top_particle = local.get_particle(top_position);
    let top_type = top_particle.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let top_state = top_particle.as_ref().map(|p| p.state).unwrap_or(ParticleState::Unknown);

    let bottom_particle = local.get_particle(bottom_position);
    let bottom_type = bottom_particle.as_ref().map(|p| p.r#type).unwrap_or(usize::MAX);
    let bottom_state = bottom_particle.as_ref().map(|p| p.state).unwrap_or(ParticleState::Unknown);

    if top_particle.is_some() {
        let top_definition = &definitions[top_type];
        if top_state == ParticleState::Fluid && center_definition.density < top_definition.density {
            center_velocity -= gravity * delta;
            center_velocity = center_velocity.max(Vec2::ZERO);
        }
    }

    if bottom_particle.is_some() {
        let bottom_definition = &definitions[bottom_type];
        if bottom_state == ParticleState::Fluid && center_definition.density > bottom_definition.density {
            center_velocity += gravity * delta;
            center_velocity = center_velocity.min(Vec2::ZERO);
        } else {
            let left_neighbour: Option<&ParticleData> = local.get_particle(center_particle.position + IVec2::new(-1, -1));
            let right_neighbour: Option<&ParticleData> = local.get_particle(center_particle.position + IVec2::new(1, -1));

            let bottom_velocity = bottom_particle.as_ref().map(|p| p.velocity).unwrap_or(Vec2::ZERO);
            let left_neighbour_velocity = left_neighbour.map(|p| p.velocity).unwrap_or(Vec2::MAX);
            let right_neighbour_velocity = right_neighbour.map(|p| p.velocity).unwrap_or(Vec2::MAX);
            let max = bottom_velocity.max(left_neighbour_velocity).max(right_neighbour_velocity);

            center_velocity += gravity * delta;

            if max.x.abs() < center_velocity.x.abs() {
                center_velocity = max;
            }
            if max.y.abs() < center_velocity.y.abs() {
                center_velocity = max;
            }
        }
    } else {
        center_velocity += gravity * delta;
    }

    local.chunks[0].get_storage_mut(state).get_unchecked_mut(particle_id).velocity = center_velocity;

    if center_velocity != Vec2::ZERO {
        local.mark_chunk_as_active(center_position);
    }
}
