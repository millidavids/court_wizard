#import bevy_pbr::forward_io::VertexOutput

struct SmokeParticleUniforms {
    color: vec4<f32>,
};

@group(3) @binding(0)
var<uniform> material: SmokeParticleUniforms;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // Radial distance from center (0 at center, 1 at edges)
    let radial = length(in.uv - vec2<f32>(0.5, 0.5)) * 2.0;

    // Soft circular falloff — fully opaque center, fades to transparent at edges
    let falloff = 1.0 - smoothstep(0.4, 1.0, radial);

    return vec4<f32>(material.color.rgb, material.color.a * falloff);
}
