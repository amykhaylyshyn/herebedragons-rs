struct Data {
    model_view_proj: mat4x4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
};

@group(0)
@binding(0)
var<uniform> r_data: Data;

@vertex
fn vs_entity(
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>
) -> VertexOutput {
    var result: VertexOutput;
    result.position = r_data.model_view_proj * vec4<f32>(pos, 1.0);
    result.normal = normal;
    return result;
}

@fragment
fn fs_entity(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>((vertex.normal + vec3<f32>(1.0)) * 0.5, 1.0);
}