use glam::{Mat4, Quat, Vec3};

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
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale(self.scale)
            * Mat4::from_quat(self.rotation)
            * Mat4::from_translation(self.position)
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::Z)
    }

    pub fn left(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::X)
    }

    pub fn up(&self) -> Vec3 {
        self.rotation.mul_vec3(Vec3::Y)
    }
}
