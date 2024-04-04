use super::chunk;
use super::chunk::Chunk;
use super::chunk::ParticleData;
use super::simulation::PowderSimulation;
use glam::IVec2;
use glam::Vec4;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockWriteGuard;

pub struct LocalChunksArcs<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub chunks: Vec<Arc<RwLock<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>>,
}

pub struct LocalChunksGuards<'a, const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub chunks: Vec<RwLockWriteGuard<'a, Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>,
}

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> LocalChunksArcs<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn new(simulation: &PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>, chunk_position: IVec2) -> Self {
        let mut chunks = Vec::new();
        let offsets = [
            IVec2::new(0, -1),
            IVec2::new(0, 1),
            IVec2::new(-1, 0),
            IVec2::new(1, 0),
            IVec2::new(-1, -1),
            IVec2::new(1, -1),
            IVec2::new(1, 1),
            IVec2::new(-1, 1),
        ];

        chunks.push(simulation.chunks[&chunk_position].clone());

        for offset in offsets {
            if let Some(chunk) = simulation.chunks.get(&(chunk_position + offset)) {
                chunks.push(chunk.clone());
            }
        }

        Self { chunks }
    }
}

impl<'a, const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>
    LocalChunksGuards<'a, CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>
{
    pub fn new(simulation: &'a LocalChunksArcs<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>) -> Self {
        let mut chunks = Vec::new();

        for chunk in &simulation.chunks {
            chunks.push(chunk.write().unwrap());
        }

        Self { chunks }
    }

    pub fn get_particle(&self, position: IVec2) -> Option<&ParticleData> {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter() {
            if chunk.position == chunk_position {
                return chunk.get_particle(position);
            }
        }

        None
    }

    pub fn get_particle_mut(&mut self, position: IVec2) -> Option<&mut ParticleData> {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter_mut() {
            if chunk.position == chunk_position {
                return chunk.get_particle_mut(position);
            }
        }

        None
    }

    pub fn add_particle(&mut self, position: IVec2, particle: ParticleData) {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter_mut() {
            if chunk.position == chunk_position {
                chunk.add_particle(position, particle);
                return;
            }
        }
    }

    pub fn remove_particle(&mut self, position: IVec2) -> Option<ParticleData> {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter_mut() {
            if chunk.position == chunk_position {
                return chunk.remove_particle(position);
            }
        }

        None
    }

    pub fn set_particle_color(&mut self, position: IVec2, color: Vec4) {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter_mut() {
            if chunk.position == chunk_position {
                chunk.set_particle_color(position, color);
                return;
            }
        }
    }

    pub fn is_position_valid(&self, position: IVec2) -> bool {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter() {
            if chunk.position == chunk_position {
                return true;
            }
        }

        false
    }

    pub fn mark_chunk_as_active(&mut self, position: IVec2) {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter_mut() {
            if chunk.position == chunk_position {
                chunk.active = true;
                return;
            }
        }
    }
}
