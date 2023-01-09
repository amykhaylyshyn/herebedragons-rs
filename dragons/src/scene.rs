use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct Material {
    pub textures: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Object {
    pub mesh: String,
    pub material_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Instance {
    pub object: String,
    pub offset: Option<(f32, f32, f32)>,
    pub scale: Option<f32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Scene {
    pub materials: HashMap<String, Material>,
    pub objects: HashMap<String, Object>,
    pub instances: Vec<Instance>,
}
