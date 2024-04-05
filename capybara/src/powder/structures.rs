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
    Particle(ParticleData),
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn create_structure(&mut self, physics: &mut PhysicsContext, points: &mut FxHashSet<IVec2>) {
        for point in points.iter() {
            let chunk = self.get_chunk(*point).unwrap();
            let mut chunk = chunk.write();

            if let Some(particle) = chunk.get_particle_mut(*point) {
                particle.structure = true;
            }
        }

        let particle_indices = points.iter().map(|p| (StructureData::Position(*p), *p)).collect::<Vec<(StructureData, IVec2)>>();
        if let Some(rigidbody_handle) = physics::create_rigidbody::<PARTICLE_SIZE, PIXELS_PER_METER>(physics, points) {
            let rigidbody = physics.rigidbodies.get(rigidbody_handle).unwrap();
            let translation = Vec2::from(rigidbody.position().translation);
            let center = translation * PIXELS_PER_METER as f32;

            let structure = Structure { rigidbody_handle, particle_indices, temporary_positions: Vec::new(), center };
            self.structures.store(Rc::new(RefCell::new(structure)));
        }
    }

    pub fn update_structures(&mut self, physics: &mut PhysicsContext) {
        let mut last_id = None;
        while let Some(id) = self.structures.get_next_id(last_id) {
            let structure = self.structures.get_unchecked(id).clone();
            let mut structure = structure.borrow_mut();

            let mut particles_to_move = Vec::new();
            let mut forbidden_for_fluid = FxHashSet::default();

            let rigidbody = physics.rigidbodies.get(structure.rigidbody_handle).unwrap();
            let position =
                Vec2::new(rigidbody.position().translation.x, rigidbody.position().translation.y) * PIXELS_PER_METER as f32 / PARTICLE_SIZE as f32;
            let rotation = rigidbody.rotation().angle();

            for p in 0..structure.particle_indices.len() {
                match structure.particle_indices[p].clone().0 {
                    StructureData::Position(position) => {
                        forbidden_for_fluid.insert(position);
                        particles_to_move.push((self.remove_particle(position).unwrap(), structure.particle_indices[p].1));
                    }
                    StructureData::Particle(particle) => {
                        forbidden_for_fluid.insert(particle.position);
                        particles_to_move.push((particle, structure.particle_indices[p].1));
                    }
                }
            }

            for p in 0..structure.temporary_positions.len() {
                self.remove_particle(structure.temporary_positions[p]).unwrap();
            }

            structure.particle_indices.clear();
            structure.temporary_positions.clear();
            let mut potential_holes = FxHashMap::default();

            for (particle, original_position) in &mut particles_to_move {
                let offset = original_position.as_vec2() - structure.center;
                let offset_after_rotation = Vec2::new(
                    offset.x * rotation.cos() - offset.y * rotation.sin(), // fmt
                    offset.x * rotation.sin() + offset.y * rotation.cos(),
                );
                let position = (position + offset_after_rotation).as_ivec2();

                if let Some(chunk) = self.get_chunk(position) {
                    let chunk = chunk.write();
                    let blocking_particle = chunk.get_particle(position);
                    let blocking_particle_state = blocking_particle.map(|p| p.state).unwrap_or(ParticleState::Unknown);

                    if blocking_particle.is_none() || blocking_particle_state == ParticleState::Fluid {
                        if blocking_particle.is_some() {
                            drop(chunk);
                            self.displace_fluid(position, &forbidden_for_fluid);
                        } else {
                            drop(chunk);
                        }

                        self.add_particle(position, *particle);

                        structure.particle_indices.push((StructureData::Position(position), *original_position));

                        for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
                            let neighbour_position = position + neighbour_offset;
                            if let Some(neighbour_chunk) = self.get_chunk(neighbour_position) {
                                let neighbour_chunk = neighbour_chunk.read();

                                let neighbour_particle = neighbour_chunk.get_particle(neighbour_position);
                                let neighbour_particle_state = neighbour_particle.map(|p| p.state).unwrap_or(ParticleState::Unknown);

                                if neighbour_particle.is_none() || neighbour_particle_state == ParticleState::Fluid {
                                    let key = neighbour_position.x | (neighbour_position.y << 16);
                                    if let Some(data) = potential_holes.get_mut(&key) {
                                        *data += 1;
                                    } else {
                                        potential_holes.insert(key, 1);
                                    }
                                }
                            }
                        }
                    } else {
                        structure.particle_indices.push((StructureData::Particle(*particle), *original_position));
                    }
                } else {
                    structure.particle_indices.push((StructureData::Particle(*particle), *original_position));
                }
            }

            for (key, filled_sides) in &potential_holes {
                if *filled_sides == 4 {
                    /*let position = IVec2::new(key & 0xffff, key >> 16);
                    let chunk = self.get_chunk(position).unwrap();
                    let chunk = chunk.read().unwrap();

                    if let Some(particle) = chunk.get_particle(position) {
                        if particle.state == ParticleState::Fluid {
                            drop(chunk);
                            self.displace_fluid(position, &forbidden_for_fluid);
                        } else {
                            drop(chunk);
                        }
                    }

                    let chunk = self.get_chunk(position + IVec2::new(0, 1)).unwrap();
                    let chunk = chunk.read().unwrap();

                    let neighbour_particle = chunk.get_particle(position + IVec2::new(0, 1));
                    if let Some(neighbour_particle) = neighbour_particle.cloned() {
                        let temporary_particle = neighbour_particle;
                        drop(chunk);

                        if !self.particle_exists(position) {
                            self.add_particle(position, temporary_particle);
                            structure.temporary_positions.push(position);
                        }
                    }*/
                }
            }

            last_id = Some(id);
        }
    }
}
