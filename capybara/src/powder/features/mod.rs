use super::chunk::Chunk;
use super::chunk::ParticleData;
use glam::IVec2;
use glam::Vec4;
use std::sync::RwLockWriteGuard;

pub mod gravity;
pub mod liquidity;
pub mod velocity;

fn get_particle<'a, const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    chunks: &'a [RwLockWriteGuard<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>],
    position: IVec2,
) -> Option<&'a ParticleData> {
    let chunk_position = get_chunk_key(position);
    for chunk in chunks.iter() {
        if chunk.position == chunk_position {
            return chunk.get_particle(position);
        }
    }

    None
}

fn get_particle_mut<'a, const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    chunks: &'a mut [RwLockWriteGuard<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>],
    position: IVec2,
) -> Option<&'a mut ParticleData> {
    let chunk_position = get_chunk_key(position);
    for chunk in chunks.iter_mut() {
        if chunk.position == chunk_position {
            return chunk.get_particle_mut(position);
        }
    }

    None
}

fn add_particle<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    chunks: &mut [RwLockWriteGuard<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>],
    position: IVec2,
    particle: ParticleData,
) {
    let chunk_position = get_chunk_key(position);
    for chunk in chunks.iter_mut() {
        if chunk.position == chunk_position {
            chunk.add_particle(position, particle);
            return;
        }
    }
}

fn remove_particle<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    chunks: &mut [RwLockWriteGuard<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>],
    position: IVec2,
) -> Option<ParticleData> {
    let chunk_position = get_chunk_key(position);
    for chunk in chunks.iter_mut() {
        if chunk.position == chunk_position {
            return chunk.remove_particle(position);
        }
    }

    None
}

fn set_particle_color<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    chunks: &mut [RwLockWriteGuard<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>],
    position: IVec2,
    color: Vec4,
) -> Option<ParticleData> {
    let chunk_position = get_chunk_key(position);
    for chunk in chunks.iter_mut() {
        if chunk.position == chunk_position {
            chunk.set_particle_color(position, color);
        }
    }

    None
}

fn is_position_valid<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    chunks: &[RwLockWriteGuard<Chunk<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER>>],
    position: IVec2,
) -> bool {
    let chunk_position = get_chunk_key(position);
    for chunk in chunks.iter() {
        if chunk.position == chunk_position {
            return true;
        }
    }

    false
}

fn get_chunk_key(position: IVec2) -> IVec2 {
    let mut chunk_position = IVec2::new(position.x >> 6, position.y >> 6);

    if position.x < 0 {
        chunk_position.x -= 1;
    }
    if position.y < 0 {
        chunk_position.y -= 1;
    }

    chunk_position
}
