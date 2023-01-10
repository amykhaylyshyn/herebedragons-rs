use glam::{Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Default::default(),
            rotation: Default::default(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform {
    pub fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale(self.scale)
            * glam::Mat4::from_quat(self.rotation)
            * glam::Mat4::from_translation(self.position)
    }
}
