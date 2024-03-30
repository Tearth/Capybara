use crate::powder::chunk::{ParticleData, ParticleState};
use crate::powder::simulation::PowderSimulation;
use glam::{IVec2, Vec2};
use std::cell::RefCell;
use std::rc::Rc;

pub fn simulate<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    simulation: &mut PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>,
    particle: Rc<RefCell<ParticleData>>,
    delta: f32,
) {
    let particle_borrow = particle.borrow_mut();
    // let definition = database.get_unchecked(particle.r#type);
    let state = particle_borrow.state;
    let mut position = particle_borrow.position;
    let mut offset = particle_borrow.offset;
    let mut velocity = particle_borrow.velocity;
    let mut velocity_budget = velocity * delta;
    drop(particle_borrow);

    while velocity_budget.length() > 0.0 {
        let step = if velocity_budget.x.abs() > velocity_budget.y.abs() {
            Vec2::new(velocity_budget.x.clamp(-1.0, 1.0), 0.0)
        } else {
            Vec2::new(0.0, velocity_budget.y.clamp(-1.0, 1.0))
        };

        offset += step;
        velocity_budget -= step;

        let position_delta = if offset.x >= 1.0 {
            IVec2::new(1, 0)
        } else if offset.x <= -1.0 {
            IVec2::new(-1, 0)
        } else if offset.y >= 1.0 {
            IVec2::new(0, 1)
        } else if offset.y <= -1.0 {
            IVec2::new(0, -1)
        } else {
            IVec2::ZERO
        };
        let position_update = position + position_delta;

        if position != position_update {
            if simulation.is_position_valid(position_update) {
                let blocking_particle = simulation.get_particle(position_update);
                let (update, swap) = if let Some(blocking_particle) = blocking_particle {
                    if state == ParticleState::Powder && blocking_particle.as_ref().borrow().state == ParticleState::Fluid {
                        (Some(position_update), true)
                    } else {
                        let neighbour_positions = if position_delta == IVec2::new(1, 0) || position_delta == IVec2::new(-1, 0) {
                            [IVec2::new(0, 1), IVec2::new(0, -1)]
                        } else if position_delta == IVec2::new(0, 1) || position_delta == IVec2::new(0, -1) {
                            [IVec2::new(1, 0), IVec2::new(-1, 0)]
                        } else {
                            panic!("Invalid particle offset")
                        };

                        let first_neighbour_position = position_update + neighbour_positions[0];
                        let second_neighbour_position = position_update + neighbour_positions[1];

                        let first_neighbour = simulation.get_particle(first_neighbour_position);
                        let second_neighbour = simulation.get_particle(second_neighbour_position);

                        let first_neighbour_slot_available = if let Some(first_neighbour) = first_neighbour.clone() {
                            first_neighbour.as_ref().borrow().state != ParticleState::Fluid
                        } else {
                            false
                        };
                        let second_neighbour_slot_available = if let Some(second_neighbour) = second_neighbour.clone() {
                            second_neighbour.as_ref().borrow().state != ParticleState::Fluid
                        } else {
                            false
                        };

                        if first_neighbour_slot_available && !second_neighbour_slot_available {
                            (Some(second_neighbour_position), second_neighbour.is_some())
                        } else if !first_neighbour_slot_available && second_neighbour_slot_available {
                            (Some(first_neighbour_position), first_neighbour.is_some())
                        } else if !first_neighbour_slot_available && !second_neighbour_slot_available {
                            if fastrand::usize(0..2) == 0 {
                                (Some(first_neighbour_position), first_neighbour.is_some())
                            } else {
                                (Some(second_neighbour_position), second_neighbour.is_some())
                            }
                        } else {
                            (None, false)
                        }
                    }
                } else {
                    (Some(position_update), false)
                };

                if let Some(position_update) = update {
                    if swap {
                        // let source_density = definition.density;
                        // let target_density = database.get_unchecked(simulation.get_particle_by_index(index_update).1.unwrap().r#type).density;
                        // let density_difference = (target_density - source_density).abs();

                        simulation.swap_particles(position, position_update);
                        velocity /= 1.9;
                    } else {
                        simulation.move_particle(position, position_update);
                    }

                    position = position_update;
                    offset %= 1.0;
                } else {
                    velocity = Vec2::ZERO;
                    offset = Vec2::ZERO;
                }
            } else {
                simulation.remove_particle(position);
                break;
            }
        }

        let mut particle = particle.borrow_mut();
        particle.position = position;
        particle.offset = offset;
        particle.velocity = velocity;
    }
}
