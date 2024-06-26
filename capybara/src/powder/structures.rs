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
    pub particles: Vec<(StructureData, Vec2, f32)>,
    pub fillings: Vec<IVec2>,
}

#[derive(Clone)]
pub enum StructureData {
    Position(IVec2),
    Particle(ParticleData),
}

impl PowderSimulation {
    pub fn create_structure(&mut self, physics: &mut PhysicsContext, points: &mut FxHashMap<IVec2, f32>) {
        let mut chunks_to_update = FxHashSet::default();

        for point in points.keys() {
            let chunk = self.get_chunk(*point).unwrap();
            let mut chunk = chunk.write();
            chunks_to_update.insert(chunk.position);

            if let Some(particle) = chunk.get_particle_mut(*point) {
                particle.structure = true;
            }
        }

        for chunk_position in &chunks_to_update {
            let chunk = self.chunks.get(chunk_position).unwrap();
            let mut chunk = chunk.write();

            chunk.update(physics);
        }

        if let Some(rigidbody_handle) = physics::create_rigidbody(physics, &mut points.clone(), self.particle_size, self.pixels_per_meter) {
            let rigidbody = physics.rigidbodies.get(rigidbody_handle).unwrap();
            let translation = Vec2::from(rigidbody.position().translation);
            let center = physics::physics_position_to_position(translation, self.particle_size, self.pixels_per_meter);
            let particles = points
                .iter()
                .map(|(p, mass)| (StructureData::Position(*p), (*p).as_vec2() + Vec2::new(0.5, 0.5) - center, *mass))
                .collect::<Vec<(StructureData, Vec2, f32)>>();

            let structure = Structure { rigidbody_handle, particles, fillings: Vec::new() };
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
            let position = physics::physics_position_to_position(rigidbody.position().translation.into(), self.particle_size, self.pixels_per_meter);
            let rotation = rigidbody.rotation().angle();
            let mut update_rigidbody = false;

            for p in 0..structure.particles.len() {
                let (_, original_position, mass) = &structure.particles[p];
                match structure.particles[p].clone().0 {
                    StructureData::Position(position) => {
                        if let Some(particle) = self.remove_particle(position) {
                            forbidden_for_fluid.insert(position);
                            particles_to_move.push((particle, *original_position, *mass));
                        } else {
                            update_rigidbody = true;
                        }
                    }
                    StructureData::Particle(particle) => {
                        forbidden_for_fluid.insert(particle.position);
                        particles_to_move.push((particle, *original_position, *mass));
                    }
                }
            }

            for p in 0..structure.fillings.len() {
                self.remove_particle(structure.fillings[p]);
            }

            structure.particles.clear();
            structure.fillings.clear();
            let mut potential_holes = FxHashMap::default();

            for (particle, original_position, mass) in &mut particles_to_move {
                let offset = *original_position;
                let offset_after_rotation = Vec2::new(
                    offset.x * rotation.cos() - offset.y * rotation.sin(), // fmt
                    offset.x * rotation.sin() + offset.y * rotation.cos(),
                );
                let position = (position + offset_after_rotation).as_ivec2();

                if let Some(chunk) = self.get_chunk(position) {
                    let chunk = chunk.write();
                    let blocking_particle = chunk.get_particle(position);
                    let blocking_particle_state = blocking_particle.map(|p| p.state).unwrap_or(ParticleState::Unknown);
                    drop(chunk);

                    if blocking_particle_state == ParticleState::Unknown || blocking_particle_state == ParticleState::Fluid {
                        if blocking_particle_state == ParticleState::Fluid {
                            self.displace_fluid(position, &forbidden_for_fluid);
                        }

                        self.add_particle(position, *particle);

                        structure.particles.push((StructureData::Position(position), *original_position, *mass));

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
                        structure.particles.push((StructureData::Particle(*particle), *original_position, *mass));
                    }
                } else {
                    structure.particles.push((StructureData::Particle(*particle), *original_position, *mass));
                }
            }

            for (key, filled_sides) in &potential_holes {
                if *filled_sides == 4 {
                    let position = IVec2::new(key & 0xffff, key >> 16);
                    let chunk = self.get_chunk(position).unwrap();
                    let chunk = chunk.read();

                    let particle = chunk.get_particle(position);
                    let particle_state = particle.map(|p| p.state).unwrap_or(ParticleState::Unknown);
                    drop(chunk);

                    if particle_state == ParticleState::Fluid {
                        self.displace_fluid(position, &forbidden_for_fluid);
                    }

                    let chunk = self.get_chunk(position + IVec2::new(0, 1)).unwrap();
                    let chunk = chunk.read();

                    let neighbour_particle = chunk.get_particle(position + IVec2::new(0, 1));
                    if let Some(neighbour_particle) = neighbour_particle.cloned() {
                        let temporary_particle = neighbour_particle;
                        drop(chunk);

                        if !self.particle_exists(position) {
                            self.add_particle(position, temporary_particle);
                            structure.fillings.push(position);
                        }
                    }
                }
            }

            if update_rigidbody {
                physics::update_rigidbody(physics, &mut structure, self.particle_size, self.pixels_per_meter);
            }

            last_id = Some(id);
        }
    }
}
