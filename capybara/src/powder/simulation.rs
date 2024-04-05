use parking_lot::RwLock;
use parking_lot::RwLockReadGuard;

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
use std::rc::Rc;
use std::sync::Arc;

pub struct PowderSimulation<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub definitions: Arc<RwLock<Vec<ParticleDefinition>>>,
    pub chunks: FxHashMap<IVec2, Arc<RwLock<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>>,
    pub structures: Storage<Rc<RefCell<Structure>>>,
    pub gravity: Vec2,

    pub debug: PowderSimulationDebugSettings,
}

pub struct PowderSimulationDebugSettings {
    pub chunk_active_color: Vec4,
    pub chunk_inactive_color: Vec4,
}

#[derive(Copy, Clone)]
pub struct ProcessData {
    gravity: Vec2,
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn logic(&mut self, renderer: &mut RendererContext, physics: &mut PhysicsContext, force_all_chunks: bool, delta: f32) {
        for (chunk_position, chunk) in &mut self.chunks {
            let mut chunk = chunk.write();

            if !chunk.initialized {
                chunk.initialize(renderer, *chunk_position);
            }

            if chunk.dirty {
                chunk.update(physics);
            }
        }

        let chunks_to_process = self.chunks.iter().filter(|p| force_all_chunks || p.1.read().active).map(|p| *p.0).collect::<Vec<_>>();
        for chunk in &mut self.chunks.values_mut() {
            chunk.write().active = false;
        }

        self.process_solid(&chunks_to_process);
        self.process_powder(&chunks_to_process, delta);
        self.process_fluid(&chunks_to_process, delta);
    }

    pub fn draw(&mut self, renderer: &mut RendererContext) {
        for chunk in &mut self.chunks.values_mut() {
            chunk.write().draw(renderer);
        }
    }

    pub fn draw_debug(&mut self, renderer: &mut RendererContext) {
        for chunk in &mut self.chunks.values_mut() {
            chunk.write().draw_debug(renderer, &self.debug);
        }
    }

    pub fn process<D, F>(&mut self, data: D, multithreaded: bool, chunks_to_process: &[IVec2], mut f: F)
    where
        D: Copy + Send + Sync,
        F: FnMut(LocalChunksGuards<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>, RwLockReadGuard<Vec<ParticleDefinition>>, D) + Copy + Send + Sync,
    {
        let offsets = [
            // Inner
            IVec2::new(-1, -1),
            IVec2::new(0, -1),
            IVec2::new(1, -1),
            IVec2::new(1, 0),
            IVec2::new(1, 1),
            IVec2::new(0, 1),
            IVec2::new(-1, 1),
            IVec2::new(-1, 0),
            // Outer
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

        let mut chunks_to_process = chunks_to_process.to_vec();
        while !chunks_to_process.is_empty() {
            let mut chunks_to_process_now = Vec::new();
            let mut chunks_available = chunks_to_process.clone();

            while !chunks_available.is_empty() {
                if let Some(chunk) = chunks_available.pop() {
                    chunks_to_process_now.push(Arc::new(RwLock::new(LocalChunksArcs::new(self, chunk))));
                    chunks_to_process.remove(chunks_to_process.iter().position(|p| *p == chunk).unwrap());
                    for offset in &offsets {
                        if let Some(index) = chunks_available.iter().position(|p| *p == chunk + *offset) {
                            chunks_available.remove(index);
                        }
                    }
                }
            }

            let mut runner = move |local: Arc<RwLock<LocalChunksArcs<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>,
                                   definitions: Arc<RwLock<Vec<ParticleDefinition>>>| {
                let local = local.write();
                let local = LocalChunksGuards::new(&local);
                let definitions = definitions.read();

                f(local, definitions, data);
            };

            if !multithreaded || chunks_to_process_now.len() == 1 {
                for local in chunks_to_process_now {
                    runner(local, self.definitions.clone());
                }
            } else {
                rayon::scope(|scope| {
                    for local in chunks_to_process_now {
                        let local = local.clone();
                        let definitions = self.definitions.clone();

                        scope.spawn(move |_| {
                            runner(local, definitions);
                        });
                    }
                });
            }
        }
    }

    pub fn process_solid(&mut self, _chunks_to_process: &[IVec2]) {}

    pub fn process_powder(&mut self, chunks_to_process: &[IVec2], delta: f32) {
        self.process(ProcessData { gravity: self.gravity }, true, chunks_to_process, |mut local, definitions, data| {
            let mut last_id = None;
            while let Some(id) = local.chunks[0].powder.get_next_id(last_id) {
                gravity::simulate(&mut local, &definitions, id, ParticleState::Powder, data.gravity, delta);
                velocity::simulate(&mut local, id, ParticleState::Powder, delta);
                last_id = Some(id);
            }
        });
    }

    pub fn process_fluid(&mut self, chunks_to_process: &[IVec2], delta: f32) {
        self.process(ProcessData { gravity: self.gravity }, true, chunks_to_process, |mut local, definitions, data| {
            let mut last_id = None;
            while let Some(id) = local.chunks[0].fluid.get_next_id(last_id) {
                gravity::simulate(&mut local, &definitions, id, ParticleState::Fluid, data.gravity, delta);
                velocity::simulate(&mut local, id, ParticleState::Fluid, delta);
                last_id = Some(id);
            }
        });

        let substep = Arc::new(RwLock::new(0));
        loop {
            let done = Arc::new(RwLock::new(true));
            self.process(ProcessData { gravity: self.gravity }, true, chunks_to_process, |mut local, definitions, _data| {
                let mut last_id = None;
                let mut needs_more_substeps = false;
                let current_substep = *substep.read();

                while let Some(id) = local.chunks[0].fluid.get_next_id(last_id) {
                    let center_particle = local.chunks[0].fluid.get_unchecked_mut(id);
                    let definition = &definitions[center_particle.r#type];

                    if current_substep < definition.fluidity {
                        liquidity::simulate(&mut local, &definitions, id);
                    }

                    if current_substep + 1 < definition.fluidity {
                        needs_more_substeps = true;
                    }

                    last_id = Some(id);
                }

                // No more fluid particles with enough fluidity to perform next substep
                if needs_more_substeps {
                    *done.write() = false;
                }
            });

            if *done.read() {
                break;
            }

            *substep.write() += 1;
        }
    }

    pub fn displace_fluid(&mut self, position: IVec2, forbidden: &FxHashSet<IVec2>) {
        let chunk = self.get_chunk(position).unwrap();
        let chunk = chunk.read();
        let particle_center = chunk.get_particle(position).expect("Particle is not a fluid");

        let particle_type = particle_center.r#type;
        let particle_hpressure = particle_center.hpressure;
        let mut available_neighbours = Vec::new();
        drop(chunk);

        for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
            let neighbour_position = position + neighbour_offset;
            let chunk = self.get_chunk(neighbour_position).unwrap();
            let chunk = chunk.write();
            let particle_neighbour = chunk.get_particle(neighbour_position);

            if forbidden.contains(&neighbour_position) {
                continue;
            }

            if let Some(particle_neighbour) = particle_neighbour {
                if particle_type == particle_neighbour.r#type {
                    available_neighbours.push((neighbour_position, false));
                }
            } else {
                available_neighbours.push((neighbour_position, true));
            }
        }

        let average_hpressure = particle_hpressure / available_neighbours.len() as f32;

        if !available_neighbours.is_empty() {
            for (neighbour_position, empty) in available_neighbours {
                let neighbour_chunk = self.get_chunk(neighbour_position).unwrap();
                let mut neighbour_chunk = neighbour_chunk.write();
                if empty {
                    neighbour_chunk.add_particle(
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
            renderer.textures.remove(chunk.read().canvas.texture_id);
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
        let mut chunk = chunk.write();

        chunk.add_particle(position, particle);
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<ParticleData> {
        let chunk = self.get_chunk_mut(position).unwrap_or_else(|| panic!("Chunk not found"));
        let mut chunk = chunk.write();

        chunk.remove_particle(position)
    }

    pub fn particle_exists(&self, position: IVec2) -> bool {
        let chunk = self.get_chunk(position);

        if let Some(chunk) = chunk {
            chunk.read().particle_exists(position)
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
        Self {
            definitions: Default::default(),
            chunks: Default::default(),
            structures: Default::default(),
            gravity: Vec2::new(0.0, -160.0),

            debug: PowderSimulationDebugSettings {
                chunk_active_color: Vec4::new(0.0, 1.0, 0.0, 1.0),
                chunk_inactive_color: Vec4::new(1.0, 0.0, 0.0, 1.0),
            },
        }
    }
}

impl ParticleData {
    pub fn present(&self) -> bool {
        self.r#type != usize::MAX
    }
}
