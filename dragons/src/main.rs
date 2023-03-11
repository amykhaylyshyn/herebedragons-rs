mod app;
mod clock;
mod ecs;
mod frame_counter;
mod scene;

use core::fmt;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fs,
    mem::size_of,
    path::Path,
};

use anyhow::Result;
use app::{Example, Spawner};
use bytemuck::{Pod, Zeroable};
use clock::{Clock, InstantClock};
use ecs::{Camera, MeshRef, Player, ShaderDataBindings, Transform};
use frame_counter::FrameCounter;
use glam::{EulerRot, Mat4, Quat, Vec2, Vec3};
use hecs::{Entity, World};
use scene::Scene;
use wgpu::util::DeviceExt;
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

// TODO: implement monkey head animation
// TODO: implement texture loading
// TODO: implement skybox
// TODO: g-buffers and deferred rendering

pub trait Dependencies: 'static {
    type Clock: Clock;
}

struct AppDependencies {}

impl Dependencies for AppDependencies {
    type Clock = InstantClock;
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    app::run::<DragonsApp<AppDependencies>>("Dragons")
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
pub struct ShaderData {
    model_view_proj: [f32; 16],
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

pub struct DragonsApp<TDeps: Dependencies> {
    world: World,
    resources: ResourceManager,
    depth_view: wgpu::TextureView,
    staging_belt: wgpu::util::StagingBelt,
    entity_pipeline: wgpu::RenderPipeline,
    pressed_keys: HashSet<VirtualKeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    camera_entity: Entity,
    view_aspect_ratio: f32,
    mouse_move_delta: Vec2,
    frame_counter: FrameCounter<TDeps::Clock>,
}

impl<TDeps: Dependencies> DragonsApp<TDeps> {
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

    fn camera_movement_system(&mut self) {
        const MOVEMENT_SPEED: f32 = 10.0;
        const MOUSE_LOOK_SPEED: f32 = std::f32::consts::FRAC_PI_4;
        let dt = self
            .frame_counter
            .avg_frame_duration()
            .unwrap_or_default()
            .as_millis() as f32
            / 1000.0;
        let query = self
            .world
            .query_mut::<(&Camera, &mut Transform, &mut Player)>();

        for (_, (_, transform, player)) in query {
            if self.pressed_mouse_buttons.contains(&MouseButton::Left) {
                player.yaw -= self.mouse_move_delta.x * MOUSE_LOOK_SPEED * dt;
                player.pitch += self.mouse_move_delta.y * MOUSE_LOOK_SPEED * dt;
                player.pitch = player
                    .pitch
                    .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
            }

            let forward = Quat::from_rotation_y(player.yaw).mul_vec3(Vec3::Z);

            let mut movement = Vec3::default();
            if self.pressed_keys.contains(&VirtualKeyCode::W) {
                movement += forward * MOVEMENT_SPEED * dt;
            }
            if self.pressed_keys.contains(&VirtualKeyCode::S) {
                movement -= forward * MOVEMENT_SPEED * dt;
            }
            if self.pressed_keys.contains(&VirtualKeyCode::D) {
                movement -= transform.left() * MOVEMENT_SPEED * dt;
            }
            if self.pressed_keys.contains(&VirtualKeyCode::A) {
                movement += transform.left() * MOVEMENT_SPEED * dt;
            }
            transform.position += movement;
            transform.rotation = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
        }
    }

    fn render_system(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<()> {
        let (camera, camera_transform) = self
            .world
            .query_one_mut::<(&Camera, &mut Transform)>(self.camera_entity)?;
        let proj_matrix = Mat4::perspective_rh(
            camera.fov_y,
            self.view_aspect_ratio,
            camera.z_near,
            camera.z_far,
        );
        let view_matrix = Mat4::look_at_rh(
            camera_transform.position,
            camera_transform.position + camera_transform.forward(),
            camera_transform.up(),
        );

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            rpass.set_pipeline(&self.entity_pipeline);

            for (_id, (transform, mesh_ref, shader_data_bindings)) in
                self.world
                    .query_mut::<(&Transform, &MeshRef, &ShaderDataBindings)>()
            {
                let model_view_proj = proj_matrix * view_matrix * transform.matrix();
                let shader_data = ShaderData {
                    model_view_proj: model_view_proj.to_cols_array(),
                };

                queue.write_buffer(
                    &shader_data_bindings.buffer,
                    0,
                    bytemuck::cast_slice(&[shader_data]),
                );
                rpass.set_bind_group(0, &shader_data_bindings.bind_group, &[]);

                let mesh_resources = self.resources.get_mesh(mesh_ref.mesh_resource_id);
                for mesh_res in mesh_resources {
                    rpass.set_index_buffer(mesh_res.indices.slice(..), wgpu::IndexFormat::Uint32);
                    rpass.set_vertex_buffer(0, mesh_res.vertices.slice(..));
                    rpass.draw_indexed(0..mesh_res.index_count as u32, 0, 0..1);
                }
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        self.staging_belt.recall();

        Ok(())
    }

    fn mouse_move_post_render_system(&mut self) {
        // reset mouse movement delta
        self.mouse_move_delta = Vec2::default();
    }
}

impl<TDeps: Dependencies> Example for DragonsApp<TDeps> {
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

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

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

            let mut transform = Transform::default();
            if let Some(offset) = instance.offset {
                transform.position = offset.into();
            }
            if let Some(scale) = instance.scale {
                transform.scale = glam::Vec3::new(scale, scale, scale);
            }

            let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance"),
                size: size_of::<ShaderData>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
                label: None,
            });

            world.spawn((
                transform,
                mesh_ref,
                ShaderDataBindings {
                    buffer: uniform_buffer,
                    bind_group,
                },
            ));
        }

        let shader_wgsl = fs::read_to_string("resources/shaders/shader.wgsl")?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_wgsl)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let entity_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Entity"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_entity",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_entity",
                targets: &[Some(config.format.into())],
            }),
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let camera_entity = {
            let camera = Camera::default();
            let mut camera_transform = Transform::default();
            camera_transform.position = Vec3::new(3.0, 1.0, -5.0);
            world.spawn((camera, camera_transform, Player::default()))
        };

        let depth_view = Self::create_depth_texture(config, &device);
        let frame_counter = Default::default();

        Ok(Self {
            world,
            depth_view,
            staging_belt: wgpu::util::StagingBelt::new(0x100),
            resources,
            entity_pipeline,
            pressed_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            camera_entity,
            view_aspect_ratio: 1.0,
            mouse_move_delta: Default::default(),
            frame_counter,
        })
    }

    fn window_event(&mut self, event: winit::event::WindowEvent) -> Result<()> {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                if let Some(virtual_key_code) = input.virtual_keycode {
                    if input.state == ElementState::Pressed {
                        if self.pressed_keys.insert(virtual_key_code) {
                            log::info!("Pressed {:?}", virtual_key_code);
                        }
                    } else {
                        if self.pressed_keys.remove(&virtual_key_code) {
                            log::info!("Released {:?}", virtual_key_code);
                        }
                    }
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                if state == ElementState::Pressed {
                    if self.pressed_mouse_buttons.insert(button) {
                        log::info!("Mouse {:?} button pressed", button);
                    }
                } else {
                    if self.pressed_mouse_buttons.remove(&button) {
                        log::info!("Mouse {:?} button released", button);
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn device_event(
        &mut self,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) -> Result<()> {
        match event {
            winit::event::DeviceEvent::MouseMotion { delta } => {
                self.mouse_move_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            _ => {}
        }
        Ok(())
    }

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.depth_view = Self::create_depth_texture(config, device);
        let aspect = config.width as f32 / config.height as f32;
        self.view_aspect_ratio = aspect;
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &Spawner,
    ) -> Result<()> {
        self.camera_movement_system();
        self.render_system(view, device, queue)?;
        self.mouse_move_post_render_system();
        self.frame_counter.frame_done();
        Ok(())
    }
}
