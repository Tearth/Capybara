use super::chunk::ParticleData;
use super::physics;
use super::simulation::PowderSimulation;
use super::ParticleState;
use crate::physics::context::PhysicsContext;
use glam::IVec2;
use glam::Vec2;
use rapier2d::dynamics::RigidBodyHandle;
use rustc_hash::FxHashMap;
use rustc_hash::FxHashSet;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
pub struct Structure {
    pub rigidbody_handle: RigidBodyHandle,
    pub particle_indices: Vec<(StructureData, IVec2)>,
    pub temporary_positions: Vec<IVec2>,
    pub center: Vec2,
}

#[derive(Clone)]
pub enum StructureData {
    Position(IVec2),
    Particle(Rc<RefCell<ParticleData>>),
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn create_structure(&mut self, physics: &mut PhysicsContext, mut points: &mut FxHashSet<IVec2>) {
        for point in points.iter() {
            if let Some(particle) = self.get_particle(*point) {
                particle.borrow_mut().structure = true;
            }
        }

        let particle_indices = points.iter().map(|p| (StructureData::Position(*p), *p)).collect::<Vec<(StructureData, IVec2)>>();
        let rigidbody_handle = physics::create_rigidbody::<PARTICLE_SIZE, PIXELS_PER_METER>(physics, &mut points);
        let rigidbody = physics.rigidbodies.get(rigidbody_handle).unwrap();
        let translation = Vec2::from(rigidbody.position().translation);
        let center = translation * PIXELS_PER_METER as f32;

        let structure = Structure { rigidbody_handle, particle_indices, temporary_positions: Vec::new(), center };
        self.structures.push(structure);

        // let rigidbody_handle = physics::create_structure::<PARTICLE_SIZE, PIXELS_PER_METER>(physics, position, &mut points);
        // let particle_indices = points.iter().map(|p| (StructureData::Position(*p), *p)).collect::<Vec<(StructureData, IVec2)>>();
        // Structure { rigidbody_handle, particle_indices, temporary_positions: Vec::new(), center: position.as_vec2() }
    }

    pub fn update_structures(&mut self, physics: &mut PhysicsContext) {
        for s in 0..self.structures.len() {
            let mut particles_to_move = Vec::new();
            let mut forbidden_for_fluid = FxHashSet::default();

            let rigidbody = physics.rigidbodies.get(self.structures[0].rigidbody_handle).unwrap();
            let position =
                Vec2::new(rigidbody.position().translation.x, rigidbody.position().translation.y) * PIXELS_PER_METER as f32 / PARTICLE_SIZE as f32;
            let rotation = rigidbody.rotation().angle();

            for p in 0..self.structures[s].particle_indices.len() {
                match self.structures[s].particle_indices[p].clone().0 {
                    StructureData::Position(position) => {
                        forbidden_for_fluid.insert(position);
                        particles_to_move.push((self.remove_particle(position).unwrap(), self.structures[s].particle_indices[p].1));
                    }
                    StructureData::Particle(particle) => {
                        forbidden_for_fluid.insert(particle.as_ref().borrow().position);
                        particles_to_move.push((particle, self.structures[s].particle_indices[p].1));
                    }
                }
            }

            for p in 0..self.structures[s].temporary_positions.len() {
                self.remove_particle(self.structures[s].temporary_positions[p]).unwrap();
            }

            self.structures[s].particle_indices.clear();
            self.structures[s].temporary_positions.clear();
            let mut potential_holes = FxHashMap::default();

            for particle in &mut particles_to_move {
                let (particle, original_position) = particle.clone();

                let offset = original_position.as_vec2() - self.structures[s].center;
                let offset_after_rotation = Vec2::new(
                    offset.x * rotation.cos() - offset.y * rotation.sin(), // fmt
                    offset.x * rotation.sin() + offset.y * rotation.cos(),
                );
                let position = (position + offset_after_rotation).as_ivec2();

                let blocking_particle = self.get_particle(position);
                let blocking_particle_state = blocking_particle.clone().map(|p| p.as_ref().borrow().state).unwrap_or(ParticleState::Unknown);

                if blocking_particle.is_none() || blocking_particle_state == ParticleState::Fluid {
                    if blocking_particle.is_some() {
                        self.displace_fluid(position, &forbidden_for_fluid);
                    }

                    self.add_particle(position, particle);
                    self.structures[s].particle_indices.push((StructureData::Position(position), original_position));

                    for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
                        let neighbour_position = position + neighbour_offset;
                        let neighbour_particle = self.get_particle(neighbour_position);
                        let neighbour_particle_state =
                            neighbour_particle.clone().map(|p| p.as_ref().borrow().state).unwrap_or(ParticleState::Unknown);

                        if neighbour_particle.is_none() || neighbour_particle_state == ParticleState::Fluid {
                            let key = neighbour_position.x | (neighbour_position.y << 16);
                            if let Some(data) = potential_holes.get_mut(&key) {
                                *data += 1;
                            } else {
                                potential_holes.insert(key, 1);
                            }
                        }
                    }
                } else {
                    self.structures[s].particle_indices.push((StructureData::Particle(particle), original_position));
                }
            }

            for (key, filled_sides) in &potential_holes {
                if *filled_sides == 4 {
                    let position = IVec2::new(key & 0xffff, key >> 16);
                    if let Some(particle) = self.get_particle(position) {
                        if particle.as_ref().borrow().state == ParticleState::Fluid {
                            self.displace_fluid(position, &forbidden_for_fluid);
                        }
                    }

                    let neighbour_particle = self.get_particle(position + IVec2::new(0, 1));
                    if let Some(neighbour_particle) = neighbour_particle {
                        let temporary_particle = neighbour_particle;
                        if !self.particle_exists(position) {
                            self.add_particle(position, temporary_particle);
                            self.structures[s].temporary_positions.push(position);
                        }
                    }
                }
            }
        }
    }
}
