mod app;
mod ecs;
mod scene;

use core::fmt;
use std::{collections::HashMap, fs, path::Path};

use anyhow::Result;
use app::{Example, Spawner};
use bytemuck::{Pod, Zeroable};
use ecs::{Camera, MeshRef, Transform};
use hecs::{Entity, World};
use scene::Scene;
use wgpu::util::DeviceExt;

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    app::run::<DragonsApp>("Dragons")
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
}

#[derive(Debug)]
pub struct MeshResources {
    name: String,
    vertices: wgpu::Buffer,
    vertex_count: usize,
    indices: wgpu::Buffer,
    index_count: usize,
}

impl fmt::Display for MeshResources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MeshResources(\"{}\")", self.name)
    }
}

impl Drop for MeshResources {
    fn drop(&mut self) {
        log::info!("Drop {}", self);
    }
}

#[derive(Debug, Default)]
pub struct ResourceManager {
    meshes: Vec<Vec<MeshResources>>,
}

impl ResourceManager {
    /// Add mesh and return mesh id that can be used to retrieve it.
    pub fn add_mesh(&mut self, mesh: Vec<MeshResources>) -> usize {
        self.meshes.push(mesh);
        self.meshes.len() - 1
    }

    pub fn get_mesh(&self, index: usize) -> &Vec<MeshResources> {
        &self.meshes[index]
    }
}

pub struct RendererState {
    pub active_camera: Entity,
}

pub struct DragonsApp {
    world: World,
    renderer_state: RendererState,
    depth_view: wgpu::TextureView,
    staging_belt: wgpu::util::StagingBelt,
    resources: ResourceManager,
}

impl DragonsApp {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

    fn create_depth_texture(
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}

impl Example for DragonsApp {
    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> Result<Self> {
        let mut resources = ResourceManager::default();
        let mut world = World::default();
        let scene_ron = fs::read_to_string("resources/scene.ron")?;
        let scene: Scene = ron::from_str(&scene_ron)?;

        let mut mesh_map: HashMap<String, usize> = HashMap::default();
        for instance in scene.instances.iter() {
            let resource_id = mesh_map.get(&instance.object);
            let mesh_ref = if let Some(resource_id) = resource_id {
                MeshRef {
                    mesh_resource_id: *resource_id,
                }
            } else {
                let obj = scene.objects.get(&instance.object).unwrap();
                let material = scene.materials.get(&obj.material_name).unwrap();

                let obj_path = Path::new("resources").join(&obj.mesh);
                let (models, materials) = tobj::load_obj(&obj_path, &tobj::GPU_LOAD_OPTIONS)?;
                materials.expect("Failed to load MTL file");

                let mesh_buffers = models
                    .into_iter()
                    .map(|model| {
                        let mut vertices: Vec<Vertex> = Vec::new();
                        let indices = &model.mesh.indices;

                        let positions = &model.mesh.positions;
                        let normals = &model.mesh.normals;
                        let texcoords = &model.mesh.texcoords;

                        for i in 0..model.mesh.positions.len() / 3 {
                            vertices.push(Vertex {
                                position: [
                                    positions[i * 3],
                                    positions[i * 3 + 1],
                                    positions[i * 3 + 2],
                                ],
                                normal: [normals[i * 3], normals[i * 3 + 1], normals[i * 3 + 2]],
                                tex_coord: [texcoords[i * 2], texcoords[i * 2 + 1]],
                            })
                        }

                        let vertices_buf =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Vertex"),
                                contents: bytemuck::cast_slice(&vertices),
                                usage: wgpu::BufferUsages::VERTEX,
                            });
                        let indices_buf =
                            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: Some("Index"),
                                contents: bytemuck::cast_slice(indices),
                                usage: wgpu::BufferUsages::INDEX,
                            });
                        MeshResources {
                            name: format!("{}/{}", obj_path.to_str().unwrap(), model.name),
                            vertices: vertices_buf,
                            vertex_count: vertices.len(),
                            indices: indices_buf,
                            index_count: indices.len(),
                        }
                    })
                    .collect::<Vec<_>>();

                let resource_id = resources.add_mesh(mesh_buffers);
                mesh_map.insert(instance.object.clone(), resource_id);

                MeshRef {
                    mesh_resource_id: resource_id,
                }
            };

            let transform = instance
                .offset
                .map(|offset| {
                    let mut transform = Transform::default();
                    transform.position = offset.into();
                    transform
                })
                .unwrap_or_default();
            world.spawn((transform, mesh_ref));
        }

        let camera_entity = world.spawn((Transform::default(), Camera::default()));
        let renderer_state = RendererState {
            active_camera: camera_entity,
        };

        let depth_view = Self::create_depth_texture(config, &device);

        Ok(Self {
            world,
            renderer_state,
            depth_view,
            staging_belt: wgpu::util::StagingBelt::new(0x100),
            resources,
        })
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {}

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.depth_view = Self::create_depth_texture(config, device);
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &Spawner,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        queue.submit(std::iter::once(encoder.finish()));

        self.staging_belt.recall();
    }
}
