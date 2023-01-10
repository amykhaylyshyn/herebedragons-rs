#[derive(Debug)]
pub struct MeshRef {
    pub mesh_resource_id: usize,
}

#[derive(Debug)]
pub struct ShaderDataBindings {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}
