#import bevy_pbr::forward_io::VertexOutput

struct ShadowUniforms {
    color: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: ShadowUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return material.color;
}
