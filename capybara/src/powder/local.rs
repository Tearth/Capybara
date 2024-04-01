use super::chunk::Chunk;
use super::chunk::ParticleData;
use super::chunk::{self};
use super::simulation::PowderSimulation;
use glam::IVec2;
use glam::Vec4;
use std::sync::RwLockWriteGuard;

pub struct LocalChunks<'a, const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> {
    pub chunks: Vec<RwLockWriteGuard<'a, Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>>,
}

impl<'a, const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> LocalChunks<'a, CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn new(simulation: &'a PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>, chunk_position: IVec2) -> Self {
        let mut chunks = Vec::new();

        chunks.push(simulation.chunks[&chunk_position].write().unwrap());
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(0, -1))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(0, 1))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(1, 0))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(-1, 0))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(1, -1))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(-1, -1))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(1, 1))) {
            chunks.push(chunk.write().unwrap());
        }
        if let Some(chunk) = simulation.chunks.get(&(chunk_position + IVec2::new(-1, 1))) {
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

    pub fn set_particle_color(&mut self, position: IVec2, color: Vec4) -> Option<ParticleData> {
        let chunk_position = chunk::get_chunk_key(position);
        for chunk in self.chunks.iter_mut() {
            if chunk.position == chunk_position {
                chunk.set_particle_color(position, color);
            }
        }

        None
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
}
