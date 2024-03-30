use self::chunk::Chunk;
use self::chunk::ParticleData;
use self::features::gravity;
use self::features::liquidity;
use self::features::velocity;
use self::structures::Structure;
use super::*;
use crate::error_return;
use crate::glam::IVec2;
use crate::glam::Vec2;
use crate::glam::Vec4;
use crate::physics::context::PhysicsContext;
use crate::renderer::context::RendererContext;
use crate::rustc_hash::FxHashMap;
use crate::rustc_hash::FxHashSet;
use crate::utils::storage::Storage;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::sync::RwLock;

pub struct PowderSimulation<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub definitions: Rc<RwLock<Vec<ParticleDefinition>>>,
    pub chunks: FxHashMap<IVec2, Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>,
    pub structures: Storage<Rc<RefCell<Structure>>>,

    pub gravity: Vec2,
    pub particles_count: u32,
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn logic(&mut self, renderer: &mut RendererContext, physics: &mut PhysicsContext, delta: f32) {
        self.process_solid();
        self.process_powder(delta);
        self.process_fluid(delta);

        for (chunk_position, chunk) in &mut self.chunks {
            if !chunk.initialized {
                chunk.initialize(renderer, *chunk_position);
            }

            if chunk.dirty {
                chunk.update(physics);
            }
        }
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        for chunk in &mut self.chunks.values_mut() {
            chunk.draw(renderer);
        }
    }

    pub fn process_solid(&mut self) {}

    pub fn process_powder(&mut self, delta: f32) {
        let definitions = self.definitions.clone();
        let definitions = definitions.read().unwrap();

        for key in self.chunks.keys().cloned().collect::<Vec<_>>() {
            let mut last_id = None;
            while let Some(id) = self.chunks[&key].powder.get_next_id(last_id) {
                let particle: &mut ParticleData = unsafe { mem::transmute(self.chunks.get_mut(&key).unwrap().powder.get_unchecked_mut(id)) };

                gravity::simulate(self, &definitions, particle, delta);
                velocity::simulate(self, particle, delta);
                last_id = Some(id);
            }
        }
    }

    pub fn process_fluid(&mut self, delta: f32) {
        let definitions = self.definitions.clone();
        let definitions = definitions.read().unwrap();

        for key in self.chunks.keys().cloned().collect::<Vec<_>>() {
            let mut last_id = None;
            while let Some(id) = self.chunks[&key].fluid.get_next_id(last_id) {
                let particle: &mut ParticleData = unsafe { mem::transmute(self.chunks.get_mut(&key).unwrap().fluid.get_unchecked_mut(id)) };

                gravity::simulate(self, &definitions, particle, delta);
                velocity::simulate(self, particle, delta);
                last_id = Some(id);
            }

            let mut current_substep = 0;
            let mut processed_particles = 0;

            loop {
                let mut last_id = None;
                while let Some(id) = self.chunks[&key].fluid.get_next_id(last_id) {
                    let center_particle: &mut ParticleData =
                        unsafe { mem::transmute(self.chunks.get_mut(&key).unwrap().fluid.get_unchecked_mut(id)) };
                    let definition = &definitions[center_particle.r#type];

                    if current_substep < definition.fluidity {
                        liquidity::simulate(self, &definitions, center_particle);
                        processed_particles += 1;
                    }

                    last_id = Some(id);
                }

                // No more fluid particles with enough fluidity to perform next substep
                if processed_particles == 0 {
                    break;
                }

                current_substep += 1;
                processed_particles = 0;
            }
        }
    }

    pub fn displace_fluid(&mut self, position: IVec2, forbidden: &FxHashSet<IVec2>) {
        let particle_center = self.get_particle(position).expect("Particle is not a fluid");

        let particle_type = particle_center.r#type;
        let mut available_neighbours = Vec::new();

        for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
            let neighbour_position = particle_center.position + neighbour_offset;
            let particle_neighbour = self.get_particle(neighbour_position);

            if forbidden.contains(&neighbour_position) {
                continue;
            }

            if let Some(particle_neighbour) = particle_neighbour {
                if particle_center.r#type == particle_neighbour.r#type {
                    available_neighbours.push((neighbour_position, false));
                }
            } else {
                available_neighbours.push((neighbour_position, true));
            }
        }

        let average_hpressure = particle_center.hpressure / available_neighbours.len() as f32;

        if !available_neighbours.is_empty() {
            for (neighbour_position, empty) in available_neighbours {
                if empty {
                    self.add_particle(
                        neighbour_position,
                        ParticleData {
                            r#type: particle_type,
                            state: ParticleState::Fluid,
                            color: Vec4::new(0.0, 0.0, 0.0, 1.0),
                            hpressure: average_hpressure,
                            ..Default::default()
                        },
                    );
                } else {
                    self.get_particle_mut(neighbour_position).unwrap().hpressure += average_hpressure;
                }
            }
        }

        self.remove_particle(position);
    }

    pub fn reset(&mut self, renderer: &mut RendererContext, physics: &mut PhysicsContext) {
        for structure in self.structures.iter() {
            physics.rigidbodies.remove(
                structure.borrow().rigidbody_handle,
                &mut physics.island_manager,
                &mut physics.colliders,
                &mut physics.impulse_joints,
                &mut physics.multibody_joints,
                true,
            );
        }

        // TODO: clear colliders
        // TODO: free canvas textures

        for chunk_position in self.chunks.keys().cloned().collect::<Vec<IVec2>>() {
            self.remove_chunk(renderer, chunk_position);
        }

        self.chunks.clear();
        self.structures.clear();
    }

    pub fn add_chunk(&mut self, chunk_position: IVec2) {
        if self.chunks.contains_key(&chunk_position) {
            error_return!("Chunk with position {} already exists", chunk_position);
        }

        self.chunks.insert(chunk_position, Chunk::default());
    }

    pub fn remove_chunk(&mut self, renderer: &mut RendererContext, chunk_position: IVec2) {
        if let Some(chunk) = self.chunks.get(&chunk_position) {
            renderer.textures.remove(chunk.canvas.texture_id);
            self.chunks.remove(&chunk_position);
        } else {
            error_return!("Chunk with position {} does not exists", chunk_position);
        }
    }

    pub fn get_chunk(&self, position: IVec2) -> Option<&Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>> {
        self.chunks.get(&(self.get_chunk_key(position)))
    }

    pub fn get_chunk_mut(&mut self, position: IVec2) -> Option<&mut Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>> {
        self.chunks.get_mut(&(self.get_chunk_key(position)))
    }

    fn get_chunk_key(&self, position: IVec2) -> IVec2 {
        let mut chunk_position = IVec2::new(position.x >> 6, position.y >> 6);

        if position.x < 0 {
            chunk_position.x -= 1;
        }
        if position.y < 0 {
            chunk_position.y -= 1;
        }

        chunk_position
    }

    pub fn add_particle(&mut self, position: IVec2, particle: ParticleData) {
        if self.get_chunk_mut(position).is_none() {
            self.add_chunk(self.get_chunk_key(position));
        }

        self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found")).add_particle(position, particle);
        self.particles_count += 1;
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<ParticleData> {
        if let Some(particle) = self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found")).remove_particle(position) {
            self.particles_count -= 1;
            Some(particle)
        } else {
            None
        }
    }

    pub fn move_particle(&mut self, from_position: IVec2, to_position: IVec2) {
        let particle = self.remove_particle(from_position).unwrap_or_else(|| panic!("Particle not found"));
        self.add_particle(to_position, particle);
    }

    pub fn swap_particles(&mut self, from_position: IVec2, to_position: IVec2) {
        let from_particle = self.remove_particle(from_position).unwrap_or_else(|| panic!("Particle not found"));
        let to_particle = self.remove_particle(to_position).unwrap_or_else(|| panic!("Particle not found"));

        self.add_particle(from_position, to_particle);
        self.add_particle(to_position, from_particle);
    }

    pub fn particle_exists(&self, position: IVec2) -> bool {
        if let Some(chunk) = self.get_chunk(position) {
            chunk.particle_exists(position)
        } else {
            false
        }
    }

    pub fn get_particle(&self, position: IVec2) -> Option<&ParticleData> {
        self.get_chunk(position).and_then(|p| p.get_particle(position))
    }

    pub fn get_particle_mut(&mut self, position: IVec2) -> Option<&mut ParticleData> {
        self.get_chunk_mut(position).and_then(|p| p.get_particle_mut(position))
    }

    pub fn set_particle_color(&mut self, position: IVec2, color: Vec4) {
        self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found")).set_particle_color(position, color);
    }

    pub fn is_position_valid(&self, position: IVec2) -> bool {
        self.get_chunk(position).is_some()
    }
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> Default
    for PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>
{
    fn default() -> Self {
        Self {
            definitions: Default::default(),
            chunks: Default::default(),
            structures: Default::default(),
            gravity: Vec2::new(0.0, -160.0),
            particles_count: Default::default(),
        }
    }
}

impl ParticleData {
    pub fn present(&self) -> bool {
        self.r#type != usize::MAX
    }
}
