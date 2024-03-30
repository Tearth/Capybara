use super::simulation::PowderSimulation;
use super::structures::StructureData;
use super::ParticleState;
use crate::physics::context::PhysicsContext;
use glam::IVec2;
use glam::Vec2;
use rapier2d::dynamics::RigidBodyBuilder;
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::Collider;
use rapier2d::geometry::ColliderBuilder;
use rapier2d::geometry::SharedShape;
use rustc_hash::FxHashSet;

impl<const CHUNK_SIZE: i32, const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32> PowderSimulation<CHUNK_SIZE, PARTICLE_SIZE, PIXELS_PER_METER> {
    pub fn apply_forces(&mut self, physics: &mut PhysicsContext) {
        let mut last_id = None;
        while let Some(id) = self.structures.get_next_id(last_id) {
            let structure = self.structures.get_unchecked(id).borrow();
            let rigidbody = physics.rigidbodies.get_mut(structure.rigidbody_handle).unwrap();

            for p in 0..structure.particle_indices.len() {
                if let StructureData::Position(position) = structure.particle_indices[p].0 {
                    let mut hpressure = Vec2::ZERO;
                    let particle = self.get_particle(position).unwrap();

                    for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
                        let neighbour_position = particle.position + neighbour_offset;
                        let neighbour_particle = self.get_particle(neighbour_position);
                        let neighbour_particle_state = neighbour_particle.map(|p| p.state).unwrap_or(ParticleState::Unknown);
                        let neighbour_particle_hpressure = neighbour_particle.map(|p| p.hpressure).unwrap_or(0.0);

                        if neighbour_particle_state == ParticleState::Fluid {
                            hpressure += -neighbour_offset.as_vec2() * neighbour_particle_hpressure;
                        }
                    }

                    if hpressure.length() > 1.0 {
                        let position = particle.position.as_vec2() * PARTICLE_SIZE as f32 + PARTICLE_SIZE as f32 / 2.0;
                        rigidbody.apply_impulse_at_point((hpressure * 0.08).into(), (position / PIXELS_PER_METER as f32).into(), true);
                    }
                }
            }

            last_id = Some(id);
        }
    }
}

pub fn create_rigidbody<const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    physics: &mut PhysicsContext,
    points: &mut FxHashSet<IVec2>,
) -> RigidBodyHandle {
    let rigidbody = RigidBodyBuilder::dynamic().build();
    let collider = self::create_collider::<PARTICLE_SIZE, PIXELS_PER_METER>(points).unwrap();
    let rigidbody_handle = physics.rigidbodies.insert(rigidbody);
    physics.colliders.insert_with_parent(collider, rigidbody_handle, &mut physics.rigidbodies);

    rigidbody_handle
}

pub fn create_collider<const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(points: &mut FxHashSet<IVec2>) -> Option<Collider> {
    let mut areas = Vec::new();
    let mut shapes = Vec::new();

    while let Some(origin) = points.iter().cloned().next() {
        let mut size = IVec2::new(1, 1);
        loop {
            if points.contains(&IVec2::new(origin.x + size.x, origin.y)) {
                size.x += 1;
            } else {
                break;
            }
        }

        'outer: loop {
            for x in origin.x..origin.x + size.x {
                for y in origin.y..origin.y + size.y + 1 {
                    if !points.contains(&IVec2::new(x, y)) {
                        break 'outer;
                    }
                }
            }

            size.y += 1;
        }

        for x in origin.x..origin.x + size.x {
            for y in origin.y..origin.y + size.y {
                points.remove(&IVec2::new(x, y));
            }
        }

        areas.push((origin, size));
    }

    if areas.is_empty() {
        return None;
    }

    let particle_size = PARTICLE_SIZE as f32 / PIXELS_PER_METER as f32;

    for (position, size) in areas {
        let half = size.as_vec2() * particle_size / 2.0;
        let cuboid = SharedShape::cuboid(half.x, half.y);
        let offset = (position.as_vec2() + size.as_vec2() / 2.0) * particle_size;
        shapes.push((offset.into(), cuboid));
    }

    Some(ColliderBuilder::compound(shapes).build())
}
