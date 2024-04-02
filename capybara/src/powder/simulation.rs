use self::chunk::Chunk;
use self::chunk::ParticleData;
use self::features::gravity;
use self::features::liquidity;
use self::features::velocity;
use self::local::LocalChunksArcs;
use self::local::LocalChunksGuards;
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
use std::sync::Arc;
use std::sync::RwLock;

pub struct PowderSimulation<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub definitions: Arc<RwLock<Vec<ParticleDefinition>>>,
    pub chunks: FxHashMap<IVec2, Arc<RwLock<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>>,
    pub structures: Storage<Rc<RefCell<Structure>>>,

    pub gravity: Vec2,
}

#[derive(Copy, Clone)]
pub struct ProcessData {
    gravity: Vec2,
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn logic(&mut self, renderer: &mut RendererContext, physics: &mut PhysicsContext, delta: f32) {
        for (chunk_position, chunk) in &mut self.chunks {
            let mut chunk = chunk.write().unwrap();

            if !chunk.initialized {
                chunk.initialize(renderer, *chunk_position);
            }

            if chunk.dirty {
                chunk.update(physics);
            }
        }

        self.process_solid();
        self.process_powder(delta);
        self.process_fluid(delta);
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        for chunk in &mut self.chunks.values_mut() {
            chunk.write().unwrap().draw(renderer);
        }
    }

    pub fn process<D, F>(&mut self, data: D, mut f: F)
    where
        D: Copy + Send + Sync,
        F: FnMut(Arc<RwLock<LocalChunksArcs<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>, Arc<RwLock<Vec<ParticleDefinition>>>, D)
            + Copy
            + Send
            + Sync,
    {
        let mut chunks_left_to_process = self.chunks.keys().cloned().collect::<Vec<_>>();
        let offsets = [
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(1, 0),
            IVec2::new(1, 1),
            IVec2::new(0, 1),
            IVec2::new(-1, 1),
            IVec2::new(-1, 0),
            //
            IVec2::new(-2, -2),
            IVec2::new(-1, -2),
            IVec2::new(0, -2),
            IVec2::new(1, -2),
            IVec2::new(2, -2),
            IVec2::new(2, -1),
            IVec2::new(2, 0),
            IVec2::new(2, 1),
            IVec2::new(2, 2),
            IVec2::new(1, 2),
            IVec2::new(0, 2),
            IVec2::new(-1, 2),
            IVec2::new(-2, 2),
            IVec2::new(-2, 1),
            IVec2::new(-2, 0),
            IVec2::new(-2, -1),
        ];

        while !chunks_left_to_process.is_empty() {
            let mut chunks_to_process = Vec::new();
            let mut chunks_available = chunks_left_to_process.clone();

            while !chunks_available.is_empty() {
                if let Some(chunk) = chunks_available.pop() {
                    chunks_to_process.push(Arc::new(RwLock::new(LocalChunksArcs::new(self, chunk))));
                    chunks_left_to_process.remove(chunks_left_to_process.iter().position(|p| *p == chunk).unwrap());
                    for offset in &offsets {
                        if let Some(index) = chunks_available.iter().position(|p| *p == chunk + *offset) {
                            chunks_available.remove(index);
                        }
                    }
                }
            }

            rayon::scope(|scope| {
                for local in chunks_to_process {
                    let local = local.clone();
                    let definitions = self.definitions.clone();

                    scope.spawn(move |_| {
                        f(local, definitions, data);
                    });
                }
            });
        }
    }

    pub fn process_solid(&mut self) {}

    pub fn process_powder(&mut self, delta: f32) {
        self.process(ProcessData { gravity: self.gravity }, |local, definitions, data| {
            let mut last_id = None;
            let local = local.write().unwrap();
            let mut local = LocalChunksGuards::new(&local);
            let definitions = definitions.read().unwrap();

            while let Some(id) = local.chunks[0].powder.get_next_id(last_id) {
                let particle: &mut ParticleData = unsafe { mem::transmute(local.chunks[0].powder.get_unchecked_mut(id)) };

                gravity::simulate(&mut local, &definitions, particle, data.gravity, delta);
                velocity::simulate(&mut local, particle, delta);
                last_id = Some(id);
            }
        });
    }

    pub fn process_fluid(&mut self, delta: f32) {
        self.process(ProcessData { gravity: self.gravity }, |local, definitions, data| {
            let mut last_id = None;
            let local = local.write().unwrap();
            let mut local = LocalChunksGuards::new(&local);
            let definitions = definitions.read().unwrap();

            while let Some(id) = local.chunks[0].fluid.get_next_id(last_id) {
                let particle: &mut ParticleData = unsafe { mem::transmute(local.chunks[0].fluid.get_unchecked_mut(id)) };

                gravity::simulate(&mut local, &definitions, particle, data.gravity, delta);
                velocity::simulate(&mut local, particle, delta);
                last_id = Some(id);
            }

            let mut current_substep = 0;
            let mut processed_particles = 0;

            loop {
                let mut last_id = None;
                while let Some(id) = local.chunks[0].fluid.get_next_id(last_id) {
                    let center_particle: &mut ParticleData = unsafe { mem::transmute(local.chunks[0].fluid.get_unchecked_mut(id)) };
                    let definition = &definitions[center_particle.r#type];

                    if current_substep < definition.fluidity {
                        liquidity::simulate(&mut local, &definitions, center_particle);
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
        });
    }

    pub fn displace_fluid(&mut self, position: IVec2, forbidden: &FxHashSet<IVec2>) {
        let chunk = self.get_chunk(position).unwrap();
        let chunk = chunk.read().unwrap();
        let particle_center = chunk.get_particle(position).expect("Particle is not a fluid");

        let particle_type = particle_center.r#type;
        let mut available_neighbours = Vec::new();

        for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
            let neighbour_position = particle_center.position + neighbour_offset;
            let particle_neighbour = chunk.get_particle(neighbour_position);

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
                let neighbour_chunk = self.get_chunk(neighbour_position).unwrap();
                let mut neighbour_chunk = neighbour_chunk.write().unwrap();
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
                    neighbour_chunk.get_particle_mut(neighbour_position).unwrap().hpressure += average_hpressure;
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

        self.chunks.insert(chunk_position, Arc::new(RwLock::new(Chunk::default())));
    }

    pub fn remove_chunk(&mut self, renderer: &mut RendererContext, chunk_position: IVec2) {
        if let Some(chunk) = self.chunks.get(&chunk_position) {
            renderer.textures.remove(chunk.read().unwrap().canvas.texture_id);
            self.chunks.remove(&chunk_position);
        } else {
            error_return!("Chunk with position {} does not exists", chunk_position);
        }
    }

    pub fn get_chunk(&self, position: IVec2) -> Option<Arc<RwLock<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>> {
        self.chunks.get(&(chunk::get_chunk_key(position))).cloned()
    }

    pub fn get_chunk_mut(&mut self, position: IVec2) -> Option<Arc<RwLock<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>> {
        self.chunks.get_mut(&(chunk::get_chunk_key(position))).cloned()
    }

    pub fn add_particle(&mut self, position: IVec2, particle: ParticleData) {
        if self.get_chunk_mut(position).is_none() {
            self.add_chunk(chunk::get_chunk_key(position));
        }

        let chunk = self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found"));
        let mut chunk = chunk.write().unwrap();

        chunk.add_particle(position, particle);
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<ParticleData> {
        let chunk = self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found"));
        let mut chunk = chunk.write().unwrap();

        chunk.remove_particle(position)
    }

    pub fn particle_exists(&self, position: IVec2) -> bool {
        let chunk = self.get_chunk(position);

        if let Some(chunk) = chunk {
            chunk.write().unwrap().particle_exists(position)
        } else {
            false
        }
    }

    pub fn is_position_valid(&self, position: IVec2) -> bool {
        self.get_chunk(position).is_some()
    }
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> Default
    for PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>
{
    fn default() -> Self {
        Self { definitions: Default::default(), chunks: Default::default(), structures: Default::default(), gravity: Vec2::new(0.0, -160.0) }
    }
}

impl ParticleData {
    pub fn present(&self) -> bool {
        self.r#type != usize::MAX
    }
}
