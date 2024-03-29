use glam::IVec2;
use rapier2d::geometry::Collider;
use rapier2d::geometry::ColliderBuilder;
use rapier2d::geometry::SharedShape;
use rustc_hash::FxHashSet;

pub fn create_collider<const PARTICLE_SIZE: i32, const PIXELS_PER_METER: i32>(mut points: FxHashSet<IVec2>) -> Collider {
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

    for (position, size) in areas {
        let particle_size = PARTICLE_SIZE as f32 / PIXELS_PER_METER as f32;
        let half = size.as_vec2() * particle_size / 2.0;
        let cuboid = SharedShape::cuboid(half.x, half.y);
        let offset = (position.as_vec2() + size.as_vec2() / 2.0) * particle_size;
        shapes.push((offset.into(), cuboid));
    }

    ColliderBuilder::compound(shapes).build()
}
