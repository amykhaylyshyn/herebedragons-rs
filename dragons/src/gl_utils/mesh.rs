use std::{fmt, path::Path};

use anyhow::Result;
use glutin::context::PossiblyCurrentContext;

use super::primitives::{Buffer, BufferType};

#[repr(C)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
}

pub struct Mesh<'a> {
    vertices: Buffer<'a>,
    indices: Buffer<'a>,
}

impl<'a> Mesh<'a> {
    pub fn from_obj<P: AsRef<Path> + fmt::Debug>(
        context: &'a PossiblyCurrentContext,
        path: P,
    ) -> Result<Vec<Self>> {
        let (models, materials_result) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
        _ = materials_result?;

        let mut meshes = Vec::with_capacity(models.len());
        for model in models.into_iter() {
            let mesh = model.mesh;
            let mut vertices = Vec::with_capacity(mesh.positions.len() / 3);
            for i in 0..mesh.positions.len() / 3 {
                let v1 = i * 3;
                let v2 = i * 3 + 1;
                let v3 = i * 3 + 2;
                let vertex = Vertex {
                    position: [mesh.positions[v1], mesh.positions[v2], mesh.positions[v3]],
                    normal: [mesh.normals[v1], mesh.normals[v2], mesh.normals[v3]],
                    tex_coord: [mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]],
                };
                vertices.push(vertex);
            }

            let vertices = Buffer::new(
                context,
                BufferType::ArrayBuffer,
                vertices.len(),
                vertices.as_ptr().cast(),
            );
            let indices = Buffer::new(
                context,
                BufferType::ElementArrayBuffer,
                mesh.indices.len(),
                mesh.indices.as_ptr().cast(),
            );
            meshes.push(Self::new(vertices, indices));
        }
        Ok(meshes)
    }

    pub fn new(vertices: Buffer<'a>, indices: Buffer<'a>) -> Self {
        Self { vertices, indices }
    }
}
