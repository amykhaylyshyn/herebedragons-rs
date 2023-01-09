use glam::{Quat, Vec3};

#[derive(Debug, Default, Clone)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
}
