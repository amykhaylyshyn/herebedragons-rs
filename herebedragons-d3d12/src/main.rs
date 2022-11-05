mod error;
mod gfx;
mod hresult;
mod renderer;

use anyhow::Result;
use dotenv::dotenv;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::{
    fs,
    io::{self, AsyncReadExt},
};

#[derive(Debug, Deserialize)]
struct Material {
    pub textures: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct Model {
    pub mesh: String,
    pub textures: HashMap<String, String>,
}

#[derive(Debug, Default, Deserialize)]
struct Point3(f32, f32, f32);

#[derive(Debug, Deserialize)]
struct Object {
    pub mesh: String,
    pub material_name: String,
    #[serde(default)]
    pub offset: Point3,
}

#[derive(Debug, Deserialize)]
struct Scene {
    pub materials: HashMap<String, Material>,
    pub objects: Vec<Object>,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();

    let mut f = fs::File::open("resources/scene.ron").await?;
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).await?;

    let scene: Scene = ron::from_str(std::str::from_utf8(&buf)?)?;
    println!("{:?}", scene);

    Ok(())
}
