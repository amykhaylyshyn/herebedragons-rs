use std::{fmt, path::Path};

use anyhow::Result;
use image::RgbaImage;

#[derive(Debug)]
pub struct ImageLibrary {
    pub dragon_texture_ao_specular_reflection: RgbaImage,
    pub dragon_texture_color: RgbaImage,
    pub dragon_texture_normal: RgbaImage,
    pub plane_texture_color: RgbaImage,
    pub plane_texture_depthmap: RgbaImage,
    pub plane_texture_normal: RgbaImage,
    pub suzanne_texture_ao_specular_reflection: RgbaImage,
    pub suzanne_texture_color: RgbaImage,
    pub suzanne_texture_normal: RgbaImage,
}

#[derive(Debug)]
pub struct Model(Vec<tobj::Model>);

pub struct ModelLibrary {
    pub dragon: Model,
    pub plane: Model,
    pub suzanne: Model,
}

pub async fn load_model<P: AsRef<Path> + fmt::Debug + Send + Sync + 'static>(
    path: P,
) -> Result<Model> {
    let (model, materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
    _ = materials?;
    Ok(Model(model))
}

pub async fn load_image<P: AsRef<Path> + Send + Sync + 'static>(path: P) -> Result<RgbaImage> {
    tokio::task::spawn_blocking(move || {
        Ok::<_, anyhow::Error>(image::io::Reader::open(path)?.decode()?.into_rgba8())
    })
    .await?
}
