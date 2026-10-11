import bevy_pbr::render::forward_io::VertexOutput;

struct ShadowUniforms {
    color: vec4<f32>,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0)
var<uniform> material: ShadowUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return material.color;
}
