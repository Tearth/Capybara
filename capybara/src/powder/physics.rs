use super::simulation::PowderSimulation;
use super::structures::Structure;
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
use rustc_hash::FxHashMap;

impl PowderSimulation {
    pub fn apply_forces(&mut self, physics: &mut PhysicsContext) {
        let mut last_id = None;
        while let Some(id) = self.structures.get_next_id(last_id) {
            let structure = self.structures.get_unchecked(id).borrow();
            let rigidbody = physics.rigidbodies.get_mut(structure.rigidbody_handle).unwrap();

            for p in 0..structure.particles.len() {
                if let StructureData::Position(position) = structure.particles[p].0 {
                    let mut force = Vec2::ZERO;
                    let mut drag_average = 0.0;
                    let mut neighbours_count = 0;

                    for neighbour_offset in [IVec2::new(1, 0), IVec2::new(-1, 0), IVec2::new(0, 1), IVec2::new(0, -1)] {
                        let neighbour_position = position + neighbour_offset;
                        if let Some(neighbour_chunk) = self.get_chunk(neighbour_position) {
                            let neighbour_chunk = neighbour_chunk.read();

                            if let Some(neighbour_particle) = neighbour_chunk.get_particle(neighbour_position) {
                                let neighbour_definition = &self.definitions.read()[neighbour_particle.r#type];
                                if neighbour_particle.state == ParticleState::Fluid {
                                    force += -neighbour_offset.as_vec2() * neighbour_particle.hpressure * neighbour_definition.displacement;
                                    drag_average += neighbour_definition.drag;
                                    neighbours_count += 1;
                                }
                            }
                        }
                    }

                    drag_average /= neighbours_count as f32;

                    let position = position.as_vec2() + Vec2::new(0.5, 0.5);
                    let position = self::position_to_physics_position(position, self.particle_size, self.pixels_per_meter);
                    let drag = f32::max(1.0, rigidbody.velocity_at_point(&position.into()).magnitude() * drag_average);
                    rigidbody.apply_impulse_at_point((force * drag).into(), position.into(), true);
                }
            }

            last_id = Some(id);
        }
    }
}

pub fn create_rigidbody(
    physics: &mut PhysicsContext,
    points: &mut FxHashMap<IVec2, f32>,
    particle_size: i32,
    pixels_per_meter: i32,
) -> Option<RigidBodyHandle> {
    if let Some((collider, center)) = self::create_collider(points, None, particle_size, pixels_per_meter) {
        let rigidbody = RigidBodyBuilder::dynamic().translation(center.into()).build();
        let rigidbody_handle = physics.rigidbodies.insert(rigidbody);
        physics.colliders.insert_with_parent(collider, rigidbody_handle, &mut physics.rigidbodies);

        Some(rigidbody_handle)
    } else {
        None
    }
}

pub fn update_rigidbody(physics: &mut PhysicsContext, structure: &mut Structure, particle_size: i32, pixels_per_meter: i32) {
    let rigidbody = physics.rigidbodies.get(structure.rigidbody_handle).unwrap();
    let collider_handle = rigidbody.colliders()[0];
    let collider = physics.colliders.get_mut(collider_handle).unwrap();
    let mut points = structure
        .particles
        .iter()
        .map(|(_, position, mass)| (position.round().as_ivec2(), *mass))
        .collect::<FxHashMap<IVec2, f32>>();
    let (collider_update, _) = create_collider(&mut points, Some(Vec2::new(0.5, 0.5)), particle_size, pixels_per_meter).unwrap();

    collider.set_mass(collider_update.mass());
    collider.set_shape(collider_update.shared_shape().to_owned());
}

pub fn create_collider(
    points: &mut FxHashMap<IVec2, f32>,
    center: Option<Vec2>,
    particle_size: i32,
    pixels_per_meter: i32,
) -> Option<(Collider, Vec2)> {
    let mut areas = Vec::new();
    let mut shapes = Vec::new();
    let mut min = IVec2::MAX;
    let mut max = IVec2::MIN;

    while let Some(origin) = points.keys().cloned().next() {
        let origin_mass = points[&origin];
        let mut left_bottom = origin;
        let mut right_top = origin;

        fn is_valid(left_bottom: IVec2, right_top: IVec2, origin_mass: f32, points: &FxHashMap<IVec2, f32>) -> bool {
            for x in left_bottom.x..=right_top.x {
                for y in left_bottom.y..=right_top.y {
                    if let Some(mass) = points.get(&IVec2::new(x, y)) {
                        if *mass != origin_mass {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
            }

            true
        }

        loop {
            let mut changed = false;

            if is_valid(left_bottom + IVec2::new(-1, 0), right_top, origin_mass, points) {
                left_bottom += IVec2::new(-1, 0);
                changed = true;
            }
            if is_valid(left_bottom + IVec2::new(0, -1), right_top, origin_mass, points) {
                left_bottom += IVec2::new(0, -1);
                changed = true;
            }
            if is_valid(left_bottom, right_top + IVec2::new(1, 0), origin_mass, points) {
                right_top += IVec2::new(1, 0);
                changed = true;
            }
            if is_valid(left_bottom, right_top + IVec2::new(0, 1), origin_mass, points) {
                right_top += IVec2::new(0, 1);
                changed = true;
            }

            if !changed {
                break;
            }
        }

        for x in left_bottom.x..=right_top.x {
            for y in left_bottom.y..=right_top.y {
                points.remove(&IVec2::new(x, y));
            }
        }

        let size = right_top - left_bottom + IVec2::ONE;
        let mass = (size.x * size.y) as f32 * origin_mass;

        min = min.min(left_bottom);
        max = max.max(right_top);

        areas.push((left_bottom, size, mass));
    }

    if areas.is_empty() {
        return None;
    }

    let mut total_mass = 0.0;
    let particle_physics_size = particle_size as f32 / pixels_per_meter as f32;
    let center = center.unwrap_or(min.as_vec2() + max.as_vec2() + Vec2::ONE) * particle_physics_size / 2.0;

    for (position, size, mass) in areas {
        let half = size.as_vec2() * particle_physics_size / 2.0;
        let cuboid = SharedShape::cuboid(half.x, half.y);
        let offset = position.as_vec2() + size.as_vec2() / 2.0;
        let offset = self::position_to_physics_position(offset, particle_size, pixels_per_meter) - center;
        shapes.push((offset.into(), cuboid));

        total_mass += mass;
    }

    Some((ColliderBuilder::compound(shapes).mass(total_mass).build(), center))
}

pub fn position_to_physics_position(position: Vec2, particle_size: i32, pixels_per_meter: i32) -> Vec2 {
    position * (particle_size as f32 / pixels_per_meter as f32)
}

pub fn physics_position_to_position(position: Vec2, particle_size: i32, pixels_per_meter: i32) -> Vec2 {
    position / (particle_size as f32 / pixels_per_meter as f32)
}
