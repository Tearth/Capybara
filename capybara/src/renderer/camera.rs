use glam::Mat4;
use glam::Vec2;
use glam::Vec3;

#[derive(Clone, PartialEq)]
pub struct Camera {
    pub position: Vec2,
    pub size: Vec2,
    pub origin: CameraOrigin,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CameraOrigin {
    LeftTop,
    LeftBottom,
}

impl Camera {
    pub fn new(position: Vec2, size: Vec2, origin: CameraOrigin) -> Self {
        Self { position, size, origin }
    }

    pub fn get_center_position(&self) -> Vec2 {
        self.position + self.size / 2.0
    }

    pub fn set_center_position(&mut self, position: Vec2) {
        self.position = position - self.size / 2.0;
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        match self.origin {
            CameraOrigin::LeftTop => Mat4::orthographic_rh(0.0, self.size.x, self.size.y, 0.0, 0.1, 100.0),
            CameraOrigin::LeftBottom => Mat4::orthographic_rh(0.0, self.size.x, 0.0, self.size.y, 0.1, 100.0),
        }
    }

    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::from_translation(Vec3::new(-self.position.x, -self.position.y, -1.0))
    }

    pub fn from_window_to_screen_coordinates(&self, position: Vec2) -> Vec2 {
        Vec2::new(position.x, self.size.y - position.y)
    }

    pub fn from_window_to_world_coordinates(&self, position: Vec2) -> Vec2 {
        Vec2::new(position.x, self.size.y - position.y) + self.position
    }

    pub fn from_screen_to_window_coordinates(&self, position: Vec2) -> Vec2 {
        Vec2::new(position.x, self.size.y - position.y)
    }

    pub fn from_world_to_window_coordinates(&self, position: Vec2) -> Vec2 {
        Vec2::new(position.x, self.size.y - position.y) - self.position
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Vec2::ZERO, Vec2::ZERO, CameraOrigin::LeftBottom)
    }
}
