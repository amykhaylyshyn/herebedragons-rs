#[derive(Debug, Clone)]
pub struct Camera {
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            fov_y: std::f32::consts::FRAC_PI_3,
            z_near: 1.0,
            z_far: 1000.0,
        }
    }
}
