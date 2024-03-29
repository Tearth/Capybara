use super::simulation::Structure;
use super::simulation::StructureData;
use crate::physics::context::PhysicsContext;
use glam::IVec2;
use glam::Vec2;
use rapier2d::dynamics::RigidBodyBuilder;
use rapier2d::dynamics::RigidBodyHandle;
use rapier2d::geometry::Collider;
use rapier2d::geometry::ColliderBuilder;
use rapier2d::geometry::SharedShape;
use rustc_hash::FxHashSet;

pub fn create_structure<const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    physics: &mut PhysicsContext,
    position: IVec2,
    mut points: &mut FxHashSet<IVec2>,
) -> Structure {
    let particle_indices = points.iter().map(|p| (StructureData::Position(*p), *p)).collect::<Vec<(StructureData, IVec2)>>();
    let rigidbody_handle = self::create_rigidbody::<PARTICLE_SIZE, PIXELS_PER_METER>(physics, position, &mut points);
    let rigidbody = physics.rigidbodies.get(rigidbody_handle).unwrap();
    let translation = Vec2::from(rigidbody.position().translation);
    let center = translation * PIXELS_PER_METER as f32;

    Structure { rigidbody_handle, particle_indices, temporary_positions: Vec::new(), center }
}

pub fn create_rigidbody<const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(
    physics: &mut PhysicsContext,
    position: IVec2,
    mut points: &mut FxHashSet<IVec2>,
) -> RigidBodyHandle {
    let particle_size = PARTICLE_SIZE as f32 / PIXELS_PER_METER as f32;

    let rigidbody = RigidBodyBuilder::dynamic().build();
    let collider = self::create_collider::<PARTICLE_SIZE, PIXELS_PER_METER>(&mut points).unwrap();
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
