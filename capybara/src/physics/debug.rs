use super::context::PhysicsContext;
use crate::renderer::context::RendererContext;
use crate::renderer::shape::Shape;
use glam::Vec2;
use rapier2d::prelude::ShapeType;
use std::sync::TryLockResult;

impl PhysicsContext {
    pub fn draw_debug(&self, context: &mut RendererContext, pixels_per_meter: f32) {
        for (_, collider) in self.colliders.iter() {
            let position = Vec2::from(collider.position().translation) * pixels_per_meter;
            let rotation = collider.rotation().angle();
            let color = if let Some(rigidbody_handle) = collider.parent() {
                if let Some(rigidbody) = self.rigidbodies.get(rigidbody_handle) {
                    if rigidbody.is_sleeping() {
                        self.debug.sleep_color
                    } else {
                        self.debug.active_color
                    }
                } else {
                    self.debug.sleep_color
                }
            } else {
                self.debug.sleep_color
            };

            match collider.shape().shape_type() {
                ShapeType::Ball => {
                    let ball = collider.shape().as_ball().unwrap();
                    let radius = ball.radius * pixels_per_meter;
                    let direction = Vec2::from_angle(rotation);

                    context.draw_shape(&Shape::new_circle(position, radius, None, self.debug.collider_thickness, color));
                    context.draw_shape(&Shape::new_line(position, position + direction * (radius - 1.0), self.debug.collider_thickness, color));
                }
                ShapeType::Cuboid => {
                    let cuboid = collider.shape().as_cuboid().unwrap();
                    let half_size = Vec2::from(cuboid.half_extents) * pixels_per_meter;
                    let mut shape = Shape::new_frame(-half_size, half_size, self.debug.collider_thickness, color);

                    shape.position = position;
                    shape.rotation = rotation;
                    context.draw_shape(&shape);
                }
                _ => {}
            }
        }

        for (_, rigidbody) in self.rigidbodies.iter() {
            let position = Vec2::from(rigidbody.position().translation) * pixels_per_meter;
            let mass_center_position = Vec2::from(rigidbody.center_of_mass().xy()) * pixels_per_meter;
            let velocity = Vec2::from(rigidbody.linvel().xy()) / self.integration_parameters.dt;

            context.draw_shape(&Shape::new_disc(mass_center_position, self.debug.mass_center_radius, None, self.debug.mass_center_color));
            context.draw_shape(&Shape::new_line(position, position + velocity, self.debug.force_thickness, self.debug.force_color));
        }

        if let TryLockResult::Ok(contacts) = self.events.contacts.try_write() {
            for contact in contacts.iter() {
                if let Some(collider) = self.colliders.get(contact.pair.collider1) {
                    for point in contact.pair.manifolds.iter().flat_map(|p| p.contacts()) {
                        let collider_position = Vec2::from(collider.position().translation) * pixels_per_meter;
                        let contact_local_position = Vec2::from(point.local_p1) * pixels_per_meter;

                        let sin = collider.rotation().angle().sin();
                        let cos = collider.rotation().angle().cos();
                        let position = Vec2::new(
                            contact_local_position.x * cos - contact_local_position.y * sin,
                            contact_local_position.y * cos + contact_local_position.x * sin,
                        ) + collider_position;

                        context.draw_shape(&Shape::new_disc(position, self.debug.contact_radius, None, self.debug.contact_color));
                    }
                }
            }
        }
    }
}
